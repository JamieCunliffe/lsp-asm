import { workspace, WorkspaceConfiguration } from 'vscode';

export class Config {
  readonly rootSection = "lsp-asm";

  get configuration() {
    return {
      "architecture": this.get<string>("architecture"),
      "codelens": {
        "enabled_filesize": this.get<string>("codelens.filesizeThreshold"),
        "loc_enabled": this.get<string>("codelens.locEnabled")
      }
    }
  }

  private get<T>(path: string): T {
    return workspace.getConfiguration(this.rootSection).get<T>(path)!;
  }
}
