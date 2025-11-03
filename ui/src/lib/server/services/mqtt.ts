import { MQTT_CONNECTION_URL, MQTT_PASSWORD, MQTT_USERNAME } from "$env/static/private";
import { MqttClient, connect } from "mqtt";

const TAG = "[MQTT]";

class MQTTClient {
    client: MqttClient;

    constructor() {
        console.log(`${TAG} Initializing MQTT client on ${MQTT_CONNECTION_URL}`);
        this.client = connect(MQTT_CONNECTION_URL, {
            clientId: `owl_ui_${Math.random().toString(16).slice(3)}`,
            username: MQTT_USERNAME,
            password: MQTT_PASSWORD,
            clean: true,
        });

        // Events
        this.client.on('connect', this.onConnect.bind(this));
        this.client.on('error', this.onError.bind(this));
    }

    private onConnect() {
        console.log(`${TAG} Connected to MQTT broker`);
    }

    private onError(error: Error) {
        if (error.message.includes("Bad username or password") || error.message.includes("Not authorized")) {
            console.error(`${TAG} ${error.message}. Exiting...`);
            process.exit(1);
        }
        console.error(`${TAG} Error:`, error.message);
    }

    public publish(topic: string, message: string) {
        if (this.client.disconnected) {
            return "Client not connected";
        }
        console.log(`${TAG} Publishing to ${topic}: ${message}`);
        this.client.publish(topic, message);
    }
}

export const mqttClient = new MQTTClient();