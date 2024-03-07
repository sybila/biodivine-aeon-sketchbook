import { html, css, unsafeCSS, LitElement, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, query, property } from 'lit/decorators.js'
import style_less from './observations-set.less?inline'
import style_tab from 'tabulator-tables/dist/css/tabulator_simple.min.css?inline'
import {
  Tabulator,
  SortModule,
  EditModule,
  PageModule,
  FilterModule,
  SelectRowModule,
  FormatModule,
  type ColumnDefinition, type CellComponent, InteractionModule
} from 'tabulator-tables'

import { type IObservation } from '../../../util/data-interfaces'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'

@customElement('observations-set')
export default class ObservationsSet extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @property() data: IObservation[] = []
  @property() variables: string[] = []
  @query('#table-wrapper') table: HTMLElement | undefined
  tabulator: Tabulator | undefined

  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()
    Tabulator.registerModule(SortModule)
    Tabulator.registerModule(EditModule)
    Tabulator.registerModule(PageModule)
    Tabulator.registerModule(FilterModule)
    Tabulator.registerModule(SelectRowModule)
    Tabulator.registerModule(FormatModule)
    Tabulator.registerModule(InteractionModule)
  }

  protected async firstUpdated (_changedProperties: PropertyValues): Promise<void> {
    super.firstUpdated(_changedProperties)
    await this.init()
    this.tabulator?.redraw(true)
  }

  private async init (): Promise<void> {
    const dataCell = (field: string): ColumnDefinition => {
      return {
        title: field,
        field,
        editor: 'textarea',
        sorter: 'number',
        headerFilter: 'tickCross',
        hozAlign: 'center',
        headerFilterParams: { tristate: true }
      }
    }
    const columns: ColumnDefinition[] = [
      {
        title: '',
        formatter: 'rowSelection',
        titleFormatter: 'rowSelection',
        headerSort: false
      },
      {
        title: 'Name',
        field: 'name',
        width: 100,
        sorter: 'string',
        headerFilter: 'input'
      }
    ]
    this.variables.forEach(v => {
      columns.push(dataCell(v))
    })
    columns.push({
      title: '',
      formatter: (_cell, _params, _callback): string => {
        return "<button class='uk-button-small uk-button-primary'>Edit</button>"
      },
      width: 70,
      headerSort: false,
      hozAlign: 'center',
      cellClick: (_e: UIEvent, _cell: CellComponent) => {
        console.log('test', _e, _cell.getData())
        void this.editObservation(_cell.getData() as IObservation)
      }
    })
    if (this.table !== undefined) {
      this.tabulator = new Tabulator(this.table, {
        columns,
        data: this.data,
        layout: 'fitDataTable',
        responsiveLayout: false,
        pagination: true,
        renderVerticalBuffer: 300,
        sortMode: 'local',
        initialSort: [{ column: 'name', dir: 'asc' }],
        headerSort: true,
        index: 'id',
        paginationSize: 20,
        selectable: 'highlight'
      }
      )
      this.tabulator.redraw(true)
    }
  }

  private async editObservation (data: IObservation): Promise<void> {
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[data.id] !== undefined) {
      await this.dialogs[data.id]?.setFocus()
      return
    }
    const renameDialog = new WebviewWindow(`editObservation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/observations-editor/edit-observation/edit-observation.html',
      title: `Edit observation (${data.id} / ${data.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[data.id] = renameDialog
    void renameDialog.once('loaded', () => {
      void renameDialog.emit('edit_observation_update', {
        ...data
      })
    })
    void renameDialog.once('edit_observation_dialog', (event: TauriEvent<{ id: string, data: IObservation }>) => {
      this.dialogs[data.id] = undefined
      const index = this.data.findIndex(observation => observation.id === data.id)
      if (index === -1) return
      void this.tabulator?.updateRow(event.payload.id, event.payload.data)
      console.log(event.payload)
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[data.id] = undefined
    })
  }

  render (): TemplateResult {
    return html`
      <div id="table-wrapper"></div>
    `
  }
}
