import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './initial-screen.less?inline'

@customElement('initial-screen')
export class InitialScreen extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  startNewSketch (): void {
    this.dispatchEvent(new CustomEvent('start-new-sketch', {
      bubbles: true,
      composed: true,
      detail: {} // maybe include some information later
    }))
  }

  importJsonProject (): void {
    // Placeholder for import JSON project functionality
  }

  importAeonModel (): void {
    // Placeholder for import AEON model functionality
  }

  importSbmlModel (): void {
    // Placeholder for import SBML model functionality
  }

  openExampleSketch (): void {
    // Placeholder for open example sketch functionality
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="header uk-background-primary uk-margin-bottom">
          <h2 class="uk-heading-bullet uk-margin-remove-bottom ">Welcome to SketchBook</h2>
          <img src="/src/assets/logo-placeholder.png" alt="Logo" class="logo-image" />
        </div>
        
        <div class="intro-section">
          <h3>Choose starting action below!</h3>
          <p>Clicking one of the buttons will open the editor. You can:</p>
          <ul class="options">
          <li>open a new project and create your sketch from scratch</li>
          <li>load a project you saved before</li>
          <li>import PSBN model in AEON or SBML format</li>
          <li>open the prepared example sketch of the TLGL network</li>
          </ul>
        </div>
        
        <div class="button-group">
          <button @click="${this.startNewSketch}" class="action-button uk-button uk-button-large uk-button-secondary uk-margin-bottom">
            Start new sketch
          </button>
          <button @click="${this.importJsonProject}" class="action-button uk-button uk-button-large uk-button-secondary uk-margin-bottom">
            Load JSON project
          </button>
          <button @click="${this.importAeonModel}" class="action-button uk-button uk-button-large uk-button-secondary uk-margin-bottom">
            Import AEON model
          </button>
          <button @click="${this.importSbmlModel}" class="action-button uk-button uk-button-large uk-button-secondary uk-margin-bottom">
            Import SBML model
          </button>
          <button @click="${this.openExampleSketch}" class="action-button uk-button uk-button-large uk-button-secondary uk-margin-bottom">
            Open example
          </button>
        </div>
      </div>
    `
  }
}
