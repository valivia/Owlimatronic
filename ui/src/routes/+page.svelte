<script lang="ts">
    import Owl from "$lib/assets/owl.svelte";
    import { audioForm } from "./audio.remote";
    import { playAnimation } from "./mqtt.remote";

    const emotes = [
        "shocked",
        "hello",
        "sweep",
        "panic",
        "yap",
        "pick_up",
        "test",
        "stream",
    ];
</script>

<main>
    <Owl />

    <h1>Owlimatronic Control Panel</h1>

    <!-- Settings -->
    <form>
        <h2>Settings</h2>

        <label>
            Discord integration:
            <input type="checkbox" name="discord_integration" />
        </label>

        <input type="submit" value="Save Settings" />
    </form>

    <!-- Animation control -->
    <form
        {...playAnimation.enhance(async ({ form, data, submit }) => {
            await submit().catch(console.error);
        })}
    >
        <h2>Animation test</h2>

        <label>
            Select emote:
            <select name={playAnimation.field("emote")}>
                {#each emotes as emote}
                    <option value={emote}>{emote}</option>
                {/each}
            </select>
        </label>

        <button type="submit" disabled={playAnimation.pending !== 0}>
            Play Animation
        </button>
    </form>

    <!-- Audio control -->
    <form
        enctype="multipart/form-data"
        {...audioForm.enhance(async ({ form, data, submit }) => {
            await submit().catch(console.error);
        })}
    >
        <h2>Audio test</h2>

        <label>
            Audio file:
            <input
                type="file"
                name={audioForm.field("audio")}
                accept="audio/*"
            />
        </label>

        <button type="submit">Play Audio</button>
    </form>
</main>

<style lang="scss">
    :global(svg) {
        width: min(80%, 30ch);
        color: #eadbc3;
    }

    h1 {
        font-weight: 400;
    }

    main {
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-inline: auto;
        padding: 2rem 1rem;
        width: min(100%, 60ch);
    }
</style>
