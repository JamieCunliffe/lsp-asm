import { workspace, WorkspaceConfiguration } from 'vscode';

export class Config {
  readonly rootSection = "lsp-asm";

  get configuration() {
    return {
      "architecture": this.get<string>("architecture"),
      "codelens": {
        "enabledFilesize": this.get<string>("codelens.filesizeThreshold"),
        "locEnabled": this.get<string>("codelens.locEnabled")
      }
    }
  }

  private get<T>(path: string): T {
    return workspace.getConfiguration(this.rootSection).get<T>(path)!;
  }
}
