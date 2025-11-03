import { spawn } from "child_process";
import { unlink, writeFile } from "fs/promises";
import { mqttClient } from "./services/mqtt";

export async function playAudio(audio: File) {
    const inputPath = `audio/${audio.name}`;
    const outputPath = `audio/stream.pcm`;

    // Save file to disk
    console.log("Saving uploaded audio to:", inputPath);
    const buffer = Buffer.from(await audio.arrayBuffer());
    await writeFile(inputPath, buffer);

    // Convert using ffmpeg
    try {
        console.log("Converting audio to PCM format...");
        await new Promise<void>((resolve, reject) => {
            const ffmpeg = spawn('ffmpeg', [
                '-y', // overwrite output
                '-i', inputPath,
                '-ac', '1',
                '-ar', '16000',
                '-f', 's16le',
                '-c:a', 'pcm_s16le',
                outputPath,
            ]);

            ffmpeg.on('error', reject);
            ffmpeg.on('close', (code) => {
                if (code === 0) resolve();
                else reject(new Error(`ffmpeg exited with code ${code}`));
            });
        });

        console.log("Conversion complete. Cleaning up...");
        await unlink(inputPath);

        console.log("Notifying MQTT broker to play audio...");
        return mqttClient.publish("owlimatronic/event", "stream");
    } catch (error) {
        console.error("Error during ffmpeg processing:", error);
        throw error;
    }


}