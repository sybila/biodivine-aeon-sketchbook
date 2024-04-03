import { html, css, unsafeCSS, LitElement, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, query, property } from 'lit/decorators.js'
import style_less from './observations-set.less?inline'
import style_tab from '../tabulator-style.less?inline'
import { Tabulator, type ColumnDefinition, type CellComponent } from 'tabulator-tables'
import { type IObservation, type IObservationSet } from '../../../util/data-interfaces'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import { checkboxColumn, dataCell, loadTabulatorPlugins, nameColumn, tabulatorOptions } from '../tabulator-utility'

@customElement('observations-set')
export default class ObservationsSet extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @property() declare data: IObservationSet
  @query('#table-wrapper') table: HTMLElement | undefined
  tabulator: Tabulator | undefined

  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()
    loadTabulatorPlugins()
  }

  protected async firstUpdated (_changedProperties: PropertyValues): Promise<void> {
    super.firstUpdated(_changedProperties)
    await this.init()
    this.tabulator?.redraw(true)
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    console.log(_changedProperties)
    void this.tabulator?.updateOrAddData(this.data.observations)
    this.tabulator?.redraw()
  }

  private async init (): Promise<void> {
    const columns: ColumnDefinition[] = [
      checkboxColumn,
      nameColumn
    ]
    this.data.variables.forEach(v => {
      columns.push(dataCell(v))
    })
    // edit button
    columns.push({
      title: '',
      formatter: (_cell, _params, _callback): string => {
        return "<button class='uk-button-small uk-button-primary'>Edit</button>"
      },
      width: 70,
      headerSort: false,
      hozAlign: 'center',
      cellClick: (_e: UIEvent, _cell: CellComponent) => {
        void this.editObservation(_cell.getData() as IObservation)
      }
    })
    // delete button
    columns.push({
      title: '',
      formatter: (_cell, _params, _callback): string => {
        return "<button class='uk-button-small uk-button-danger'>Delete</button>"
      },
      width: 80,
      headerSort: false,
      hozAlign: 'center',
      cellClick: (_e: UIEvent, _cell: CellComponent) => {
        // todo: send through backend
        void _cell.getRow().delete()
      }
    })
    if (this.table !== undefined) {
      this.tabulator = new Tabulator(this.table, {
        ...tabulatorOptions,
        columns,
        data: this.data.observations,
        popupContainer: this.table,
        rowContextMenu: [
          {
            label: 'Add Row',
            action: () => {
              this.dispatchEvent(new CustomEvent('add-observation', {
                detail: {
                  id: this.data.id
                },
                bubbles: true,
                composed: true
              }))
              this.requestUpdate()
            }
          },
          {
            label: 'Edit Row',
            action: (_, row) => {
              void this.editObservation(row.getData() as IObservation)
            }
          },
          {
            label: 'Delete Row',
            action: function (_, row) {
              void row.delete()
            }
          }
        ]
      })
      this.tabulator.redraw(true)
    }
  }

  private async editObservation (obs: IObservation): Promise<void> {
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[obs.id] !== undefined) {
      await this.dialogs[obs.id]?.setFocus()
      return
    }
    const renameDialog = new WebviewWindow(`editObservation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/observations-editor/edit-observation/edit-observation.html',
      title: `Edit observation (${obs.id} / ${obs.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[obs.id] = renameDialog
    void renameDialog.once('loaded', () => {
      void renameDialog.emit('edit_observation_update', {
        ...obs
      })
    })
    void renameDialog.once('edit_observation_dialog', (event: TauriEvent<{ id: string, data: IObservation }>) => {
      this.dialogs[obs.id] = undefined
      const index = this.data.observations.findIndex(observation => observation.id === obs.id)
      if (index === -1) return
      void this.tabulator?.updateRow(event.payload.id, event.payload.data)
      console.log(event.payload)
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[obs.id] = undefined
    })
  }

  render (): TemplateResult {
    return html`
      <div id="table-wrapper"></div>
    `
  }
}
