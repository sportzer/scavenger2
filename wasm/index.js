const { Game } = wasm_bindgen;

const WIDTH = 80;
const HEIGHT = 36;

let display = null;
let game = null;

const update = () => {
    if (game == null) { return; }
    game.run();

    const toColor = (n) => ROT.Color.toHex([(n)&255, (n>>8)&255, (n>>16)&255]);

    if (display != null) {
        for (let x = 0; x < WIDTH; x++) {
            for (let y = 0; y < HEIGHT; y++) {
                const ch = String.fromCharCode(game.get_ch(x, y));
                const fg = toColor(game.get_fg(x, y));
                const bg = toColor(game.get_bg(x, y));
                display.draw(x, y, ch, fg, bg);
            }
        }
    }
};

const initDisplay = () => {
    window.focus()

    display = new ROT.Display();
    display.setOptions({
        width: WIDTH,
        height: HEIGHT,
        fontSize: 18,
        fontFamily: "'Ubuntu Mono', monospace",
        bg: "black",
    });
    document.body.appendChild(display.getContainer());
    document.body.focus();

    document.addEventListener("keydown", e => {
        if (game == null) { return; }
        game.push_keydown_event(e.keyCode, e.ctrlKey, e.altKey, e.shiftKey);
        // TODO: figure out something which is less of a hack
        // (only block arrows here and space in keypress handler?)
        if(e.key.length != 1) {
            e.preventDefault();
        }
        requestAnimationFrame(update);
    });
    document.addEventListener("keypress", e => {
        if (game == null) { return; }
        game.push_keypress_event(e.charCode, e.ctrlKey, e.altKey);
        e.preventDefault();
        requestAnimationFrame(update);
    });

    display.getContainer().addEventListener("mousedown", e => {
        if (game == null) { return; }
        const pos = display.eventToPosition(e);
        game.push_mouse_press_event(pos[0], pos[1], e.button);
        requestAnimationFrame(update);
    });
    display.getContainer().addEventListener("mouseup", e => {
        if (game == null) { return; }
        const pos = display.eventToPosition(e);
        game.push_mouse_release_event(pos[0], pos[1], e.button);
        requestAnimationFrame(update);
    });

    let minDelta = 1e9;
    display.getContainer().addEventListener("wheel", e => {
        minDelta = Math.min(minDelta, Math.abs(e.deltaY));
        if (game == null) { return; }
        const pos = display.eventToPosition(e);
        game.push_mouse_wheel_event(pos[0], pos[1], Math.round(e.deltaY/minDelta));
        e.preventDefault();
        requestAnimationFrame(update);
    });

    requestAnimationFrame(update);
};

wasm_bindgen('./scavenger_wasm_bg.wasm').then(() => {
    const seed = Math.floor(Math.random() * Math.pow(2, 32));
    // console.log("game seed: " + seed);
    game = Game.new(seed);
    game.set_size(WIDTH, HEIGHT);
    requestAnimationFrame(update);
});
