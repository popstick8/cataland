import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

const root = document.getElementById("root");
if (!root) throw new Error("Missing application root");

createRoot(root).render(
	<StrictMode>
		<main>
			<h1>Cataland</h1>
		</main>
	</StrictMode>,
);
