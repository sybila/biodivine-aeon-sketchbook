import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './observations-editor.less?inline'
import './observations-set/observations-set'
import { ContentData, type IObservation, type IObservationSet, DataCategory } from '../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import { dialog } from '@tauri-apps/api'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import { basename } from '@tauri-apps/api/path'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../util/config'

@customElement('observations-editor')
export default class ObservationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()
  @state() datasets: IObservationSet[] = []

  constructor () {
    super()
    this.addEventListener('add-observation', this.addObservation)
  }

  private addObservation (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log(detail)
    const setIndex = this.datasets.findIndex(dataset => dataset.id === detail.id)
    if (setIndex === -1) return
    this.datasets[setIndex].observations.push(this.singleDummy(this.datasets[setIndex].observations.length))
    this.datasets = [...this.datasets]
    console.log(this.datasets)
  }

  getDummy = (): IObservation[] => Array(10).fill(0).map((_, index) => {
    return this.singleDummy(index)
  })

  private singleDummy (index: number): IObservation {
    const ret: IObservation = {
      id: String(index).padStart(4, '0'),
      name: 'obs' + String(index).padStart(4, '0')
    }
    this.contentData.variables.forEach(v => {
      ret[v.name] = Math.round(Math.random())
    })
    return ret
  }

  private async import (): Promise<void> {
    const handle = await dialog.open({
      title: 'Import observation set...',
      multiple: false,
      filters: [
        {
          name: 'Comma-separated values',
          extensions: ['csv']
        },
        {
          name: 'Tab-separated values',
          extensions: ['tsv', 'tab']
        },
        {
          name: 'All',
          extensions: ['*']
        }
      ]
    })
    if (handle === null) return
    let fileName
    if (Array.isArray(handle)) {
      fileName = handle.pop() ?? 'unknown'
    } else {
      fileName = handle
    }

    // TODO: allow proper import in future
    const name = await basename(fileName)
    void this.importObservations(name, this.getDummy(), this.contentData.variables.map(v => v.name))
  }

  private async importObservations (name: string, data: IObservation[], variables: string[]): Promise<void> {
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    const importDialog = new WebviewWindow(`editObservation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/observations-editor/observations-import/observations-import.html',
      title: 'Import observation set',
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    void importDialog.once('loaded', () => {
      void importDialog.emit('observations_import_update', {
        data,
        variables: this.contentData.variables.map(v => v.name)
      })
    })
    void importDialog.once('observations_import_dialog', (event: TauriEvent<IObservation[]>) => {
      this.datasets = this.datasets.concat({
        id: name,
        observations: event.payload,
        variables,
        category: DataCategory.UNSPECIFIED
      })
    })
  }

  updateSetName = debounce((name: string, id: number) => {
    this.datasets[id].id = name
  }, functionDebounceTimer
  )

  render (): TemplateResult {
    return html`
      <div class="observations">
        <div class="header">
          <div></div>
          <h1 class="heading uk-heading-line uk-text-center">Observations</h1>
          <button @click="${this.import}" class="uk-button uk-button-primary uk-button-small import-button">+ Import</button>
        </div>
        <div class="accordion-body">
          <div class="accordion">
            ${map(this.datasets, (dataset, index) => html`
          <div class="container" id="${'container' + index}">
            <div class="label" @click="${() => { this.shadowRoot?.getElementById('container' + index)?.classList.toggle('active') }}" >
              <input 
                  @input="${(e: InputEvent) => {
                    this.updateSetName((e.target as HTMLInputElement).value, index)
                  }}"
                  ?readonly="${true}"
                  @dblclick="${(e: InputEvent) => {
                    (e.target as HTMLInputElement).readOnly = !(e.target as HTMLInputElement).readOnly
                  }}"
                  class="set-name heading uk-input uk-form-blank"
                  value="${dataset.id}"/>
            </div>
            <div class="content">
              <observations-set
                  .data="${dataset}">
              </observations-set>
            </div>
          </div>
          <hr>
        `)}
          </div>
        </div>
      </div>
      
    `
  }
}
