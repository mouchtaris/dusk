import * as path from "path";
import {
  workspace,
  ExtensionContext,
} from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Executable,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext) {
  const config = workspace.getConfiguration("dust.lsp");
  const enabled = config.get<boolean>("enabled", true);
  if (!enabled) {
    return;
  }

  const command = config.get<string>("path", "dust-lsp");

  const serverExecutable: Executable = {
    command,
    args: [],
  };

  const serverOptions: ServerOptions = {
    run: serverExecutable,
    debug: serverExecutable,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "dust" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.dust"),
    },
  };

  client = new LanguageClient(
    "dust-lsp",
    "Dust Language Server",
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
