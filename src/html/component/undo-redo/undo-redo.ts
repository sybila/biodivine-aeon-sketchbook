import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './undo-redo.less?inline'
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
import { aeonState } from '../../../aeon_events'
library.add(faArrowLeft, faArrowRight)

@customElement('undo-redo')
class UndoRedo extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @state()
  private can_undo: boolean = false

  @state()
  private can_redo: boolean = false

  constructor () {
    super()
    aeonState.undo_stack.can_undo.addEventListener((it) => {
      this.can_undo = it
    })
    aeonState.undo_stack.can_redo.addEventListener((it) => {
      this.can_redo = it
    })
  }

  render (): TemplateResult {
    return html`
      <div class="undo-redo uk-flex-nowrap">
        <button class="uk-button uk-button-secondary uk-button-small"
                @click=${aeonState.undo_stack.undo} ?disabled=${!this.can_undo}>${icon(findIconDefinition({ prefix: 'fas', iconName: 'arrow-left' })).node}</button>
        <button class="uk-button uk-button-secondary uk-button-small"
                @click=${aeonState.undo_stack.redo} ?disabled=${!this.can_redo}>${icon(findIconDefinition({ prefix: 'fas', iconName: 'arrow-right' })).node}</button>
      </div>
    `
  }
}
