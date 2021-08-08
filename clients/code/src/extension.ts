import * as lc from 'vscode-languageclient/node';
import { commands, workspace, ExtensionContext } from 'vscode';
import { Config } from './config'
import { openLoc } from './notifications'

import {
  LanguageClient,
  LanguageClientOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(_context: ExtensionContext) {
  const newEnv = Object.assign({}, process.env);

  const run: lc.Executable = {
        command: "lsp-asm",
        options: { env: newEnv },
    };

  const serverOptions: lc.ServerOptions = {
        run,
        debug: run,
    };

  let config = new Config();
  let clientOptions: LanguageClientOptions = {
    initializationOptions: config.configuration,
    documentSelector: [{ scheme: 'file', language: 'asm' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/.s')
    }
  };

  commands.registerCommand("lsp-asm.loc", openLoc);

  client = new LanguageClient(
    'lsp-asm',
    'lsp asm',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
