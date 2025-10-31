import { DISCORD_TOKEN } from "$env/static/private";
import { Client, GatewayIntentBits, Events, Message } from "discord.js";
import { mqttClient } from "./mqtt";

const TAG = "[DISCORD]";

class DiscordIntegration {
    client: Client;

    constructor() {
        console.log(`${TAG} Initializing Discord client...`);
        this.client = new Client({ intents: [GatewayIntentBits.Guilds, GatewayIntentBits.MessageContent, GatewayIntentBits.GuildMessages] });

        // Events
        this.client.once(Events.ClientReady, this.onReady.bind(this, this.client));
        this.client.on(Events.MessageCreate, this.onMessage.bind(this));

        this.client.login(DISCORD_TOKEN);
    }

    private onReady(client: Client) {
        console.log(`${TAG} Client ready! Logged in as "${client.user?.tag}"`);
    }

    private onMessage(message: Message) {
        if (message.author.bot) return;

        console.log(`${TAG} Message from ${message.author.tag}: ${message.content}`);

        mqttClient.publish("owlimatronic/event", "yap");

        return;
        const audio = message.attachments
            .map(attachment => attachment.contentType?.startsWith("audio/") ? attachment : null)
            .filter(url => url !== null);

        if (audio.length > 0) {
            console.log(`${TAG} Audio attachments found:`, audio);
        }
    }
}

export const discordClient = new DiscordIntegration();