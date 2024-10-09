import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './annotations-tab.less?inline'
import { ContentData } from '../../util/data-interfaces'

@customElement('annotations-tab')
export class AnnotationsTab extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()

  constructor () {
    super()
    console.log('debug print')
  }

  addSketchAnnot (): void {
    // TODO
  }

  formatSketchAnnotation (): TemplateResult<1> {
    return html`No annotations available for the sketch.`
  }

  addVarAnnot (): void {
    // TODO
  }

  formatVarAnnotations (): TemplateResult<1> {
    const annotatedVars = this.contentData.variables
        .filter(variable => variable.annotation.trim() !== "");
    if (annotatedVars.length === 0) {
        return html`<p>No annotations available for variables.</p>`;
    }
    return html`${annotatedVars.map(variable => html`${variable.id}: ${variable.annotation}<br>`)}`;
  }

  addFnAnnot (): void {
    // TODO
  }

  formatFnAnnotations (): TemplateResult<1> {
    const annotatedFns = this.contentData.functions
        .filter(func => func.annotation.trim() !== "");
    if (annotatedFns.length === 0) {
        return html`<p>No annotations available for functions.</p>`;
    }
    return html`${annotatedFns.map(func => html`${func.id}: ${func.annotation}<br>`)}`;
  }

  addDatasetAnnot (): void {
    // TODO
  }

  formatDatasetAnnotations (): TemplateResult<1> {
    return html`${this.contentData.observations
      .filter(dataset => dataset.annotation.trim() !== "" || dataset.observations.some(observation => observation.annotation.trim() !== ""))
      .map(dataset => html`
      <div class="dataset">
          <p>${dataset.id}: ${dataset.annotation}</p>
          <ul>
            ${dataset.observations
                .filter(observation => observation.annotation.trim() !== "")
                .map(observation => html`
                  <li>${observation.id}: ${observation.annotation}</li>
                `)}
          </ul>
      </div>
    `)}`;
  }

  addDynPropAnnot (): void {
    // TODO
  }

  formatDynPropAnnotations (): TemplateResult<1> {
    const annotatedProps = this.contentData.dynamicProperties
        .filter(dynProp => dynProp.annotation.trim() !== "");
    if (annotatedProps.length === 0) {
        return html`<p>No annotations available for dynamic properties.</p>`;
    }
    return html`${annotatedProps.map(dynProp => html`${dynProp.id}: ${dynProp.annotation}<br>`)}`;
  }

  addStatPropAnnot (): void {
    // TODO
  }

  formatStatPropAnnotations (): TemplateResult<1> {
    const annotatedProps = this.contentData.staticProperties
        .filter(dynProp => dynProp.annotation.trim() !== "");
    if (annotatedProps.length === 0) {
        return html`<p>No annotations available for static properties.</p>`;
    }
    return html`${annotatedProps.map(dynProp => html`${dynProp.id}: ${dynProp.annotation}<br>`)}`;
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="components-list">
          <div class="section" id="whole-sketch">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Sketch Annotation</h3>
              <button id="add-sketch-annot-button" class="add-annot uk-button uk-button-small uk-button-primary"
                      @click="${this.addSketchAnnot}">
                      + Add
              </button>
            </div>
            <div class="annotation">
              ${this.formatSketchAnnotation()}
            </div>
          </div>
          <div class="section" id="variables">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Variables</h3>
              <button id="add-var-annot-button" class="add-annot uk-button uk-button-small uk-button-primary"
                      @click="${this.addVarAnnot}">
                      + Add
              </button>
            </div>
            <div class="annotation">
              ${this.formatVarAnnotations()}
            </div>
          </div>
          <div class="section" id="functions">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Uninterpreted Functions</h3>
              <button id="add-fn-annot-button" class="add-annot uk-button uk-button-small uk-button-primary"
                      @click="${this.addFnAnnot}">
                      + Add
              </button>
            </div>
            <div class="annotation">
              ${this.formatFnAnnotations()}
            </div>
          </div>
          <div class="section" id="datasets">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Datasets</h3>
              <button id="add-dataset-annot-button" class="add-annot uk-button uk-button-small uk-button-primary"
                      @click="${this.addDatasetAnnot}">
                      + Add
              </button>
            </div>
            <div class="annotation">
              ${this.formatDatasetAnnotations()}
            </div>
         </div>
          <div class="section" id="static">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Static Properties</h3>
              <button id="add-stat-annot-button" class="add-annot uk-button uk-button-small uk-button-primary"
                      @click="${this.addStatPropAnnot}">
                      + Add
              </button>
            </div>
            <div class="annotation">
              ${this.formatStatPropAnnotations()}
            </div>
          </div>
          <div class="section" id="dynamic">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Dynamic Properties</h3>
              <button id="add-dyn-annot-button" class="add-annot uk-button uk-button-small uk-button-primary"
                      @click="${this.addDynPropAnnot}">
                      + Add
              </button>
            </div>
            <div class="annotation">
              ${this.formatDynPropAnnotations()}
            </div>
          </div>
        </div> 
      </div>
    `
  }
}