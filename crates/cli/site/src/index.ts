import Root from "./root.svelte";
import "./index.css";

const root = new Root({
    target: document.getElementById("root") as Element,
});

export default root;
