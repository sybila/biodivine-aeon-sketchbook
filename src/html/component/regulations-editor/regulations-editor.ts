import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './regulations-editor.less?inline'
import cytoscape, { type Core, type EdgeSingular, type NodeSingular, type Position } from 'cytoscape'
import dagre from 'cytoscape-dagre'
import edgeHandles, { type EdgeHandlesInstance } from 'cytoscape-edgehandles'
import dblclick from 'cytoscape-dblclick'
import './float-menu/float-menu'
import { edgeOptions, initOptions } from './regulations-editor.config'
import { ElementType, type Monotonicity } from './element-type'
import { dialog } from '@tauri-apps/api'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/event'
import UIkit from 'uikit'
import { type IEdgeData, type INodeData } from './graph-interfaces'
import { dummyData } from '../../util/dummy-data'
import { ContentData } from '../../util/tab-data'

const SAVE_NODES = 'nodes'
const SAVE_EDGES = 'edges'

@customElement('regulations-editor')
class RegulationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  dialogs: Record<string, WebviewWindow | undefined> = {}

  @query('#float-menu')
    nodeMenu!: HTMLElement

  @query('#edge-menu')
    edgeMenu!: HTMLElement

  editorElement
  cy: Core | undefined
  edgeHandles: EdgeHandlesInstance | undefined
  lastTabCount = 1
  @property() contentData = ContentData.create()
  @state() menuType = ElementType.NONE
  @state() menuPosition = { x: 0, y: 0 }
  @state() menuZoom = 1.0
  @state() menuData = undefined
  @state() drawMode = false

  constructor () {
    super()
    cytoscape.use(dagre)
    cytoscape.use(edgeHandles)
    cytoscape.use(dblclick)
    this.addEventListener('update-edge', this.updateEdge)
    this.addEventListener('adjust-graph', this.adjustPan)
    this.addEventListener('add-edge', this.addEdge)
    this.addEventListener('mousemove', this.hoverFix)
    this.addEventListener('remove-element', (e) => {
      void this.removeElement(e)
    })
    this.addEventListener('rename-node', (e) => {
      void this.renameNode(e)
    })
    this.addEventListener('update-function', () => { this.toggleMenu(ElementType.NONE) })

    this.editorElement = document.createElement('div')
    this.editorElement.id = 'cytoscape-editor'
  }

  render (): TemplateResult {
    return html`
      <button @click=${this.loadDummyData}
              class="uk-button uk-button-danger uk-button-small uk-margin-large-left uk-position-absolute uk-position-z-index-high">
        reset (debug)
      </button>
      ${this.editorElement}
      <float-menu .type=${this.menuType} .position=${this.menuPosition} .zoom=${this.menuZoom}
                  .data=${this.menuData}></float-menu>
    `
  }

  hoverFix (): void {
    // TODO
  }

  addEdge (event: Event): void {
    this.cy?.nodes().deselect()
    this.toggleMenu(ElementType.NONE)
    const nodeID = (event as CustomEvent).detail.id

    // start attribute wrongly typed - added weird typecast to avoid tslint error
    this.edgeHandles?.start((this.cy?.$id(nodeID) as unknown as string))
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error
    this.cy.renderer().hoverData.capture = true
  }

  firstUpdated (): void {
    this.init()
    if (!this.loadCachedNodes() || !this.loadCachedEdges()) this.loadDummyData()
  }

  init (): void {
    this.cy = cytoscape(initOptions(this.editorElement))
    this.edgeHandles = this.cy.edgehandles(edgeOptions)

    this.cy.on('add remove position ehcomplete', this.saveState)

    this.cy.on('zoom', () => {
      this.renderMenuForSelectedNode()
      this.renderMenuForSelectedEdge()
    })
    this.cy.on('pan', () => {
      this.renderMenuForSelectedNode()
      this.renderMenuForSelectedEdge()
    })
    this.cy.on('dblclick', (e) => {
      if (e.target !== this.cy) return // dont trigger when mouse is over cy elements
      const name = (Math.random() + 1).toString(36).substring(8).toUpperCase()
      this.addNode(name, name, e.position)
      this.saveState()
    })
    this.cy.on('mouseover', 'node', function (e) {
      e.target.addClass('hover')
    })

    this.cy.on('mouseover', 'node', (e) => {
      e.target.addClass('hover')
      // node.addClass('hover')
      // this._modelEditor.hoverVariable(id, true)
    })
    this.cy.on('mouseout', 'node', (e) => {
      e.target.removeClass('hover')
      // this._modelEditor.hoverVariable(id, false)
    })
    this.cy.on('select', 'node', (e) => {
      // deselect any previous selection - we don't support multiselection yet
      // this.cy?.$(':selected').forEach((selected) => {
      //   if (selected.data().id !== id) {
      //     selected.unselect()
      //   }
      // })
      this.renderMenuForSelectedNode(e.target)
      // this._modelEditor.selectVariable(id, true)
    })
    this.cy.on('unselect', 'node', () => {
      this.toggleMenu(ElementType.NONE)
      // this._modelEditor.selectVariable(id, false)
    })
    // node.on('click', () => {
    //   this._lastClickTimestamp = 0 // ensure that we cannot double-click inside the node
    // })
    this.cy.on('drag', (e) => {
      if ((e.target as NodeSingular).selected()) this.renderMenuForSelectedNode(e.target)
      this.renderMenuForSelectedEdge()
    })

    this.cy.on('select', 'edge', (e) => {
      this.renderMenuForSelectedEdge(e.target)
    })
    this.cy.on('unselect', 'edge', () => {
      this.toggleMenu(ElementType.NONE) // hide menu
    })
    this.cy.on('mouseover', 'edge', (e) => {
      e.target.addClass('hover')
      // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, true);
    })
    this.cy.on('mouseout', 'edge', (e) => {
      e.target.removeClass('hover')
      // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, false);
    })

    // this.cy.on('mousemove', (e) => {
    //   console.log(e)
    // })

    this.cy.ready(() => {
      this.cy?.center()
      this.cy?.fit(undefined, 50)
      this.cy?.resize()
    })
  }

  loadDummyData (): void {
    console.log('loading dummy data')
    this.cy?.remove('node')
    this.cy?.edges().remove()
    this.addNodes(dummyData.nodes)
    this.addEdges(dummyData.edges)
    this.saveState()

    this.cy?.ready(() => {
      this.cy?.center()
      this.cy?.fit(undefined, 50)
      this.cy?.resize()
    })
  }

  async renameNode (event: Event): Promise<void> {
    this.toggleMenu(ElementType.NONE)
    const nodeId = (event as CustomEvent).detail.id
    const nodeName = (event as CustomEvent).detail.name
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[nodeId] !== undefined) {
      await this.dialogs[nodeId]?.setFocus()
      return
    }
    const renameDialog = new WebviewWindow(`renameDialog${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/rename-dialog/rename-dialog.html',
      title: `Edit node (${nodeId} / ${nodeName})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      resizable: false,
      height: 100,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[nodeId] = renameDialog
    void renameDialog.once('loaded', () => {
      void renameDialog.emit('edit_node_update', {
        id: nodeId,
        name: nodeName
      })
    })
    void renameDialog.once('edit_node_dialog', (event: TauriEvent<{ id: string, name: string }>) => {
      this.dialogs[nodeId] = undefined
      // avoid overwriting existing nodes
      if (nodeId !== event.payload.id && (this.cy?.$id(event.payload.id) !== undefined && this.cy?.$id(event.payload.id).length > 0)) {
        UIkit.notification(`Node with id '${event.payload.id}' already exists!`)
        return
      }
      const node = this.cy?.$id(nodeId)
      if (node === undefined) return
      const position = node.position()
      const edges = this.contentData.edges.filter((edge) => edge.source === nodeId || edge.target === nodeId)
      node.remove()
      this.addNode(event.payload.id, event.payload.name, position)
      edges.forEach((edge) => {
        if (edge.source === nodeId) {
          this.ensureRegulation({ ...edge, source: event.payload.id })
        } else {
          this.ensureRegulation({ ...edge, target: event.payload.id })
        }
      })
      this.saveState()
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[nodeId] = undefined
    })
  }

  adjustPan (event: Event): void {
    const tabCount = (event as CustomEvent).detail.tabCount
    if (tabCount === this.lastTabCount) return
    const toLeft = this.lastTabCount < tabCount
    this.lastTabCount = tabCount
    this.cy?.panBy({ x: (toLeft ? -1 : 1) * (this.cy?.width() / (tabCount * 2)), y: 0 })
  }

  renderMenuForSelectedNode (node: NodeSingular | undefined = undefined): void {
    if (node === undefined) {
      node = this.cy?.nodes(':selected').first()
      if (node === undefined || node.length === 0) return // nothing selected
    }
    const zoom = this.cy?.zoom()
    const position = node.renderedPosition()
    this.toggleMenu(ElementType.NODE, position, zoom, node.data())
  }

  renderMenuForSelectedEdge (edge: EdgeSingular | undefined = undefined): void {
    if (edge === undefined) {
      edge = this.cy?.edges(':selected').first()
      if (edge === undefined || edge.length === 0) return // nothing selected
    }
    const zoom = this.cy?.zoom()
    const boundingBox = edge.renderedBoundingBox()
    const position = {
      x: (boundingBox.x1 + boundingBox.x2) / 2,
      y: (boundingBox.y1 + boundingBox.y2) / 2
    }
    this.toggleMenu(ElementType.EDGE, position, zoom, edge.data())
  }

  addNode (id: string, name: string, position = { x: 0, y: 0 }): void {
    this.cy?.add({
      data: { id, name },
      position: { ...position }
    })
  }

  toggleMenu (type: ElementType, position: Position | undefined = undefined, zoom = 1.0, data = undefined): void {
    this.menuType = type
    if (this.menuType === ElementType.NONE) this.cy?.nodes().deselect()
    this.menuPosition = position ?? { x: 0, y: 0 }
    this.menuZoom = zoom
    this.menuData = data
    this.saveState()
  }

  ensureRegulation (regulation: IEdgeData): void {
    // const currentEdge = this._findRegulationEdge(regulation.regulator, regulation.target)
    // if (currentEdge !== undefined) {
    //   // Edge exists - just make sure to update data
    //   const data = currentEdge.data()
    //   data.observable = regulation.observable
    //   data.monotonicity = regulation.monotonicity
    //   this.cy?.style().update() // redraw graph
    //   if (currentEdge.selected()) {
    //     // if the edge is selected, we also redraw the edge menu
    //     this._renderMenuForSelectedEdge(currentEdge)
    //   }
    // } else {
    // Edge does not exist - create a new one
    this.cy?.add({
      group: 'edges',
      data: {
        source: regulation.source,
        target: regulation.target,
        observable: regulation.observable,
        monotonicity: regulation.monotonicity
      }
    })
  }

  updateEdge (event: Event): void {
    const e = (event as CustomEvent)
    this.cy?.$id(e.detail.edgeId)
      .data('observable', e.detail.observable)
      .data('monotonicity', e.detail.monotonicity)
    this.menuData = this.cy?.$id(e.detail.edgeId).data()
  }

  async removeElement (event: Event): Promise<void> {
    if (!await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })) return
    const nodeId = (event as CustomEvent).detail.id
    void this.dialogs[nodeId]?.close()
    this.cy?.$id(nodeId).remove()
    this.toggleMenu(ElementType.NONE)
  }

  saveState (): void {
    const nodes = ((this.cy?.nodes()) ?? []).map((node): INodeData => {
      return {
        id: node.data().id,
        name: node.data().name,
        position: node.position()
      }
    })
    const edges: IEdgeData[] = ((this.cy?.edges()) ?? []).map((edge): IEdgeData => {
      return {
        id: edge.id(),
        source: edge.source().id(),
        target: edge.target().id(),
        observable: edge.data().observable as boolean,
        monotonicity: edge.data().monotonicity as Monotonicity
      }
    })
    if (nodes.length > 0) {
      localStorage.setItem(SAVE_NODES, JSON.stringify(nodes))
    }
    if (edges.length > 0) {
      localStorage.setItem(SAVE_EDGES, JSON.stringify(edges))
    }
    this.contentData = ContentData.create({ nodes, edges })
    this.shadowRoot?.dispatchEvent(new CustomEvent('update-data', {
      detail: {
        nodes,
        edges
      },
      composed: true,
      bubbles: true
    }))
  }

  loadCachedNodes (): boolean {
    try {
      const parsed = (JSON.parse(localStorage.getItem(SAVE_NODES) ?? '[]') as INodeData[])
      if (parsed.length === 0) return false
      this.addNodes(parsed)
    } catch (e) {
      return false
    }
    console.log('nodes loaded')
    return true
  }

  loadCachedEdges (): boolean {
    try {
      const parsed = (JSON.parse(localStorage.getItem(SAVE_EDGES) ?? '[]') as IEdgeData[])
      if (parsed.length === 0) return false
      this.addEdges(parsed)
    } catch (e) {
      return false
    }
    console.log('edges loaded')
    return true
  }

  addNodes (nodes: INodeData[]): void {
    nodes.forEach((node) => {
      this.addNode(node.id, node.name, node.position)
    })
  }

  addEdges (edges: IEdgeData[]): void {
    edges.forEach((edge) => {
      this.ensureRegulation(edge)
    })
  }
}
