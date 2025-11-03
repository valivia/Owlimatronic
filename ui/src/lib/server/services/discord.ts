import { DISCORD_CHANNEL_ID, DISCORD_TOKEN } from "$env/static/private";
import { Client, GatewayIntentBits, Events, Message } from "discord.js";
import { mqttClient } from "./mqtt";
import { playAudio } from "../util";

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

    private async onMessage(message: Message) {
        if (message.author.bot) return;
        if (message.channel.id !== DISCORD_CHANNEL_ID) return;

        console.log(`${TAG} Message from ${message.author.tag}: ${message.content}`);

        const audioFiles = message.attachments
            .map(attachment => attachment.contentType?.startsWith("audio/") ? attachment : null)
            .filter(url => url !== null);

        const attachment = audioFiles[0];
        if (attachment) {
            console.log(`${TAG} Audio attachments found`);
            const response = await fetch(attachment.url);

            const file = new File([await response.arrayBuffer()], attachment.name, { type: response.headers.get("content-type") || "application/octet-stream" });

            await playAudio(file);

        } else {
            mqttClient.publish("owlimatronic/event", "yap");
        }
    }
}

export const discordClient = new DiscordIntegration();