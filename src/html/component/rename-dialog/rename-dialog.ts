import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query } from 'lit/decorators.js'
import style_less from './rename-dialog.less?inline'
import { emit, listen, type Event as TauriEvent } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'

@customElement('rename-dialog')
class RenameDialog extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#name') nameField: HTMLInputElement | undefined
  @query('#nodeID') nodeIdField: HTMLInputElement | undefined
  nodeId = ''
  name = ''

  async connectedCallback (): Promise<void> {
    super.connectedCallback()

    await listen('edit_node_update', (event: TauriEvent<{ id: string, name: string }>) => {
      if (this.nodeIdField !== undefined) this.nodeIdField.value = event.payload.id
      if (this.nameField !== undefined) this.nameField.value = event.payload.name
    })
  }

  private async handleSubmit (event: Event): Promise<void> {
    event.preventDefault()
    if (this.nodeId === '' || this.name === '') {
      this.nameField?.classList.remove('uk-form-danger')
      this.nodeIdField?.classList.remove('uk-form-danger')
      if (this.nodeId === '') {
        this.nodeIdField?.classList.add('uk-form-danger')
        console.log('id empty')
      }
      if (this.name === '') {
        this.nameField?.classList.add('uk-form-danger')
        console.log('name empty')
      }

      return
    }
    await emit('edit_node_dialog', {
      id: this.nodeId,
      name: this.name
    })
    await appWindow.close()
  }

  private handleIdUpdate (e: Event): void {
    this.nodeId = (e.target as HTMLInputElement).value
  }

  private handleNameUpdate (e: Event): void {
    this.name = (e.target as HTMLInputElement).value
  }

  render (): TemplateResult {
    return html`
            <form class="uk-form-horizontal">
<!--                <div class="uk-h2">Edit node</div>-->
                <div class="uk-margin-small">
                    <label class="uk-form-label" for="form-horizontal-text">Node ID</label>
                    <div class="uk-form-controls">
                        <input class="uk-input" @input="${this.handleIdUpdate}" id="nodeID" type="text" placeholder="ID" value="" />
                    </div>
                </div>
                <div class="uk-margin-small">
                    <label class="uk-form-label" for="form-horizontal-text">Node Name</label>
                    <div class="uk-flex uk-flex-row">
                        <input class="uk-input" @input="${this.handleNameUpdate}" id="name" type="text" placeholder="Name" value="" />
                    </div>
                </div>
                
            
            <button class="uk-button uk-width-1-1" @click="${this.handleSubmit}">Submit</button>
            </form>
    `
  }
}