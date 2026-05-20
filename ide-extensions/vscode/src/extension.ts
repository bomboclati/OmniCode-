import * as vscode from 'vscode';
import { OmniClient } from './client';
import { ChatSidebarProvider } from './chat';
import { DiffViewProvider } from './diff';
import { StatusBarManager } from './status';

let omniClient: OmniClient | undefined;
let statusBar: StatusBarManager | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('OmniCode extension activating...');

    omniClient = new OmniClient('ws://localhost:9421/ws');
    omniClient.connect();

    const chatProvider = new ChatSidebarProvider(context.extensionUri, omniClient);
    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider('omniCodeChat', chatProvider)
    );

    const diffProvider = new DiffViewProvider();
    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider('omniCodeDiff', diffProvider)
    );

    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('omniCodeStatus', new StatusTreeProvider(omniClient))
    );

    statusBar = new StatusBarManager(omniClient);
    statusBar.initialize();

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.chat', () => {
            vscode.commands.executeCommand('workbench.view.extension.omniCodeChat');
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.swarm', async () => {
            const task = await vscode.window.showInputBox({
                prompt: 'Enter swarm task description',
                placeHolder: 'e.g., "refactor the authentication module"',
            });
            if (task && omniClient) {
                omniClient.send({ type: 'swarm', content: task });
                vscode.window.showInformationMessage(`Swarm started: ${task}`);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.review', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                const text = editor.document.getText();
                if (omniClient) {
                    omniClient.send({ type: 'review', content: text });
                }
                vscode.window.showInformationMessage('Review requested');
            } else {
                vscode.window.showWarningMessage('No active editor');
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.deploy', async () => {
            const confirmed = await vscode.window.showQuickPick(['Yes', 'No'], {
                placeHolder: 'Deploy current project?',
            });
            if (confirmed === 'Yes' && omniClient) {
                omniClient.send({ type: 'deploy' });
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.find', async () => {
            const query = await vscode.window.showInputBox({
                prompt: 'Semantic search query',
                placeHolder: 'e.g., "database connection handling"',
            });
            if (query && omniClient) {
                omniClient.send({ type: 'search', content: query });
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.heal', () => {
            if (omniClient) {
                omniClient.send({ type: 'heal' });
            }
            vscode.window.showInformationMessage('Self-healing initiated');
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.docs', () => {
            if (omniClient) {
                omniClient.send({ type: 'docs' });
            }
            vscode.window.showInformationMessage('Documentation regeneration started');
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('omnicode.openPalette', () => {
            vscode.commands.executeCommand('workbench.action.quickOpen');
        })
    );

    // Handle config changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('omnicode')) {
                const config = vscode.workspace.getConfiguration('omnicode');
                const port = config.get<number>('port', 9421);
                if (omniClient) {
                    omniClient.disconnect();
                    omniClient = new OmniClient(`ws://localhost:${port}/ws`);
                    omniClient.connect();
                }
            }
        })
    );
}

class StatusTreeProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    constructor(private client: OmniClient) {
        client.on('connected', () => this._onDidChangeTreeData.fire());
        client.on('disconnected', () => this._onDidChangeTreeData.fire());
    }
    private _onDidChangeTreeData = new vscode.EventEmitter<vscode.TreeItem | undefined>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(): vscode.TreeItem[] {
        const status = this.client.isConnected()
            ? { label: 'Connected', icon: 'pass-filled', tooltip: 'WebSocket connected' }
            : { label: 'Disconnected', icon: 'warning', tooltip: 'WebSocket disconnected' };
        const item = new vscode.TreeItem(status.label);
        item.tooltip = status.tooltip;
        return [item];
    }
}

export function deactivate() {
    if (omniClient) {
        omniClient.disconnect();
    }
    if (statusBar) {
        statusBar.dispose();
    }
}
