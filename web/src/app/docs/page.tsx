import { redirect } from "next/navigation";

/** `/docs` has no page of its own — it is an alias for the introduction. */
export default function DocsIndex() {
  redirect("/docs/getting-started");
}
