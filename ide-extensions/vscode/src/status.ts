import * as vscode from 'vscode';
import { OmniClient } from './client';

export class StatusBarManager {
    private statusBarItem: vscode.StatusBarItem;
    private interval: NodeJS.Timeout | undefined;

    constructor(private readonly client: OmniClient) {
        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Right,
            100
        );
        this.statusBarItem.command = 'omnicode.openPalette';
    }

    initialize() {
        this.statusBarItem.text = '$(debug-start) OmniCode';
        this.statusBarItem.tooltip = 'Click to open OmniCode commands';
        this.statusBarItem.show();

        this.updateStatus();

        this.interval = setInterval(() => this.updateStatus(), 2000);

        this.client.on('agent_status', (msg) => {
            this.updateStatusText(msg.content || 'idle');
        });

        this.client.on('connected', () => {
            this.statusBarItem.text = '$(link) OmniCode';
            this.statusBarItem.backgroundColor = undefined;
        });

        this.client.on('disconnected', () => {
            this.statusBarItem.text = '$(warning) OmniCode';
            this.statusBarItem.backgroundColor = new vscode.ThemeColor(
                'statusBarItem.warningBackground'
            );
        });
    }

    private updateStatus() {
        const connected = this.client.isConnected();
        if (connected) {
            this.statusBarItem.text = '$(link) OmniCode';
        }
    }

    private updateStatusText(status: string) {
        const icons: Record<string, string> = {
            idle: '$(circle-outline)',
            planning: '$(lightbulb)',
            running: '$(loading~spin)',
            swarm: '$(multiple-windows)',
            incident: '$(error)',
            guardian: '$(shield)',
        };

        const icon = icons[status] || '$(debug-start)';
        this.statusBarItem.text = `${icon} OmniCode: ${status}`;

        const colors: Record<string, string | undefined> = {
            idle: undefined,
            running: '#00FFA3',
            incident: '#FF4444',
            guardian: '#FFAA00',
        };

        this.statusBarItem.color = new vscode.ThemeColor(
            'statusBarItem.foreground'
        );
    }

    dispose() {
        if (this.interval) {
            clearInterval(this.interval);
        }
        this.statusBarItem.dispose();
    }
}
