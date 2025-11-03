import * as v from 'valibot';
import { form } from '$app/server';
import { playAudio } from "$lib/server/util";

export const audioForm = form(
    v.object({
        audio: v.file(),
    }),
    async ({ audio }) => {
        await playAudio(audio);

        return undefined;
    }
);