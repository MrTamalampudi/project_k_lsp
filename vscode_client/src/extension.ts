import * as lc from 'vscode-languageclient/node';
import * as vscode from 'vscode';
import * as os from "os";
import { existsSync, watchFile } from 'fs';

let client: lc.LanguageClient;

//testing

function getLocalPath(): string {
   let path = os.homedir() + "/Documents/rust/project_k/lsp/target/debug/lsp";
   if (existsSync(path)) {
      return path;
   }
   return "";
}

async function startServer(path: string) {
   const run: lc.Executable = {
      command: path,
      transport: lc.TransportKind.stdio,
   };
   const serverOptions: lc.ServerOptions = {
      run,
      debug: run,
   };
   const clientOptions: lc.LanguageClientOptions = {
      documentSelector: [{ scheme: 'file', language: 'project_k' }],
   };

   client = new lc.LanguageClient('project_k', 'project_k', serverOptions, clientOptions);
   

   await client.start();
}

export async function activate(context: vscode.ExtensionContext) {

   let local_path = getLocalPath();
   await startServer(local_path);
}

export async function deactivate(): Promise<void> {
   if (!client) {
      return;
   }
   return client.stop();
}