import * as v from 'valibot';
import { form } from '$app/server';
import { mqttClient } from "$lib/server/mqtt";

export const playAnimation = form(
    v.object({
        emote: v.pipe(v.string(), v.nonEmpty()),
    }),
    async ({ emote }) => {
        const result = mqttClient.publish("owlimatronic/event", emote);

        return result;
    }
);