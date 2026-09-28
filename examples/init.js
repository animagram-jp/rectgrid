const params = new URLSearchParams(location.search);
if (params.has("eruda")) {
    const s = document.createElement("script");
    s.src = "https://cdn.jsdelivr.net/npm/eruda";
    s.onload = () => eruda.init();
    document.body.appendChild(s);
}

let worker = start();

function start() {
    const w = new Worker("./worker.js", { type: "module" });

    w.addEventListener("message", (e) => {
        const { type, payload } = e.data;
        if (type === "execute") { payload.forEach(execute); }
        if (type === "error")   { worker.terminate(); worker = start(); }
    });

    w.addEventListener("error", (e) => {
        console.error("[worker] restart:", e.message);
        worker.terminate();
        worker = start();
    });

    const section_rect = document.getElementById("section")?.getBoundingClientRect();

    w.postMessage({
        type: "init",
        payload: {
            pointer_coarse:   window.matchMedia("(pointer: coarse)").matches,
            viewport_width:   window.innerWidth,
            viewport_height:  window.innerHeight,
            section_origin_x: section_rect?.left ?? 0,
            section_origin_y: section_rect?.top ?? 0,
        },
    });

    let bound = false;
    w.addEventListener("message", function on_ready(e) {
        if (e.data.type !== "ready" || bound) return;
        bound = true;
        w.removeEventListener("message", on_ready);
        bind();
    });

    return w;
}

function execute(cmd) {
    if (cmd.operation === 18) {
        console.error("[wasm]", cmd.message);
        return;
    }

    const el = document.getElementById(encode_id(cmd.id));
    if (!el) return;
    switch (cmd.operation) {
        case 1:  el.textContent = cmd.value ?? ""; break;
        case 2:  el.value = cmd.value ?? ""; break;
        case 3:  el.setAttribute(ATTRIBUTES[cmd.attribute], cmd.value ?? ""); break;
        case 4:  el.removeAttribute(ATTRIBUTES[cmd.attribute]); break;
        case 5:  el.classList.add(CLASS_NAMES[cmd.value]); break;
        case 6:  el.classList.remove(CLASS_NAMES[cmd.value]); break;
        case 7:  el.style.width = cmd.px + "px"; break;
        case 8:  el.style.height = cmd.px + "px"; break;
        case 9:  el.style.zIndex = cmd.z; break;
        case 10: el.style.background = cmd.value; break;
        case 11: el.style.translate = `${cmd.x}px ${cmd.y}px`; break;
        case 12: el.style.cursor = CURSOR_VALUES[cmd.value] ?? ""; break;
        case 13: el.showModal(); break;
        case 14: el.close(); break;
        case 15: el.focus(); break;
        case 16: js_fn[FN_NAMES[cmd.name]]?.(el); break;
    }
}

const ATTRIBUTES = ["disabled", "hidden"];

const CLASS_NAMES = ["hide", "show", "hidden", "highlighted"];

const CURSOR_VALUES = ["default", "grab", "", "nwse-resize", "nesw-resize", "ew-resize", "ns-resize"];

const FN_NAMES = ["hide_toast", "show_toast"];

const EVENT_TYPES = [
    null,
    "change",
    "click",
    "contextmenu",
    "drop",
    "focusin",
    "focusout",
    "input",
    "keydown",
    "pointercancel",
    "pointerdown",
    "pointermove",
    "pointerup",
    "resize",
    "scroll",
    "shutdown",
    "submit",
];

const KEY_NAMES = [
    null,
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "Backspace",
    "Enter",
    "Escape",
    "Tab",
];

const TAGS = [
    "",
    "article",
    "body",
    "button",
    "dd",
    "dl",
    "drawer",
    "dt",
    "fieldset",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "header",
    "input",
    "li",
    "main",
    "modal",
    "ol",
    "output",
    "p",
    "section",
    "select",
    "span",
    "table",
    "tbody",
    "td",
    "textarea",
    "th",
    "thead",
    "tr",
    "ul",
];

function decode_id(id) {
    if (!id) return [];
    return id.split("_").map((segment) => {
        const dash = segment.lastIndexOf("-");
        const number = dash < 0 ? NaN : Number(segment.slice(dash + 1));
        const tag = Number.isInteger(number) ? segment.slice(0, dash) : segment;
        return [Math.max(0, TAGS.indexOf(tag)), Number.isInteger(number) ? number : null];
    });
}

function encode_id(segments) {
    return segments
        .map(([tag, n]) => (n == null ? TAGS[tag] : `${TAGS[tag]}-${n}`))
        .join("_");
}

const js_fn = {
    show_toast: (el) => {
        el.classList.remove("hidden");
        requestAnimationFrame(() => requestAnimationFrame(() => {
            el.classList.add("show");
            setTimeout(() => {
                el.classList.replace("show", "hide");
                el.addEventListener("transitionend", () => el.classList.remove("hide"), { once: true });
            }, 3000);
        }));
    },
    hide_toast: (el) => {
        el.classList.replace("show", "hide");
        el.addEventListener("transitionend", () => el.classList.remove("hide"), { once: true });
    },
};

const ROOTS = ["header", "main", "modal", "form", "output", "section"]
    .map(id => document.getElementById(id));

function send(e) {
    if (!ROOTS.some(r => r && r.contains(e.target))) return;
    worker.postMessage({ type: "event", payload: {
        event_type: Math.max(0, EVENT_TYPES.indexOf(e.type)),
        target_id:  decode_id(e.target.id ?? ""),
        key:        Math.max(0, KEY_NAMES.indexOf(e.key)),
        value:      e.target.value ?? "",
        x:          e.clientX ?? 0,
        y:          e.clientY ?? 0,
        time:       e.timeStamp ?? 0,
        pointer_id: e.pointerId ?? 0,
    }});
}

function bind() {
    const EVENTS = ["click", "keydown", "input", "change", "submit", "focusout",
                    "pointerdown", "pointerup", "pointermove", "pointercancel"];
    for (const type of EVENTS) {
        document.addEventListener(type, send);
    }

    let resize_timer;
    window.addEventListener("resize", () => {
        clearTimeout(resize_timer);
        resize_timer = setTimeout(() => {
            const rect = document.getElementById("section")?.getBoundingClientRect();
            worker.postMessage({ type: "event", payload: {
                event_type:       EVENT_TYPES.indexOf("resize"),
                target_id:        [],
                key:              0,
                value:            "",
                x:                window.innerWidth,
                y:                window.innerHeight,
                time:             performance.now(),
                section_origin_x: rect?.left ?? 0,
                section_origin_y: rect?.top ?? 0,
            }});
        }, 100);
    });

    window.addEventListener("scroll", (e) => {
        worker.postMessage({ type: "event", payload: {
            event_type: EVENT_TYPES.indexOf("scroll"),
            target_id:  decode_id(e.target?.id ?? ""),
            x:          window.scrollX,
            y:          window.scrollY,
        }});
    }, { passive: true });

    window.addEventListener("pagehide", (e) => {
        if (e.persisted) return;
        worker.postMessage({ type: "event", payload: { event_type: EVENT_TYPES.indexOf("shutdown") } });
    });
}
