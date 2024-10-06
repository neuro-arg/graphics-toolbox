import init from "web";
import { basicSetup, EditorView } from 'codemirror';
// import { vim } from "@replit/codemirror-vim"

class Platform {
  constructor() {
    this.watchers = {};
  }
  watchFile(name, callback) {
    if (name == "shader.wgsl") {
      this.watchers[name] = x => callback(name, x);
    }
  }
  unwatchFile(name) {
    delete this.watchers[name];
  }
  listFiles() {
    return ['nuero.png', 'shader.wgsl'];
  }
  reportError(errorString) {
    console.error(error);
  }
}

export const runApp = async () => {
  try {
    globalThis.platform = new Platform();
    globalThis.editorView = new EditorView({
      doc: "",
      extensions: [
        // vim(),
        basicSetup,
        EditorView.updateListener.of(v => {
          if (v.docChanged) {
            globalThis.platform.watchers['shader.wgsl'](new TextEncoder().encode(globalThis.editorView.state.toJSON().doc));
          }
        })
      ],
      parent: document.getElementsByTagName('body')[0],
    });
    await init();
    console.log("WASM Loaded");
  } catch (e) {
    console.error(e);
  }
};

runApp();
