import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './undo-redo.less?inline'
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { aeonState } from '../../../aeon_state'
library.add(faArrowLeft, faArrowRight)

@customElement('undo-redo')
export default class UndoRedo extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @state()
  private canUndo: boolean = false

  @state()
  private canRedo: boolean = false

  constructor () {
    super()
    aeonState.undoStack.canUndo.addEventListener((it) => {
      this.canUndo = it
    })
    aeonState.undoStack.canRedo.addEventListener((it) => {
      this.canRedo = it
    })
  }

  render (): TemplateResult {
    return html`
      <div class="undo-redo uk-flex-nowrap">
        <button class="uk-button uk-button-secondary uk-button-small ${!this.canUndo ? 'disabled' : ''}"
                @click=${aeonState.undoStack.undo} ?disabled=${!this.canUndo}>${icon(faArrowLeft).node}</button>
        <button class="uk-button uk-button-secondary uk-button-small ${!this.canRedo ? 'disabled' : ''}"
                @click=${aeonState.undoStack.redo} ?disabled=${!this.canRedo}>${icon(faArrowRight).node}</button>
      </div>
    `
  }
}
