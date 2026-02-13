# Implementation Patterns

## Svelte Template Editing
When wrapping existing elements in a new parent (e.g., adding a `<div class="header">` around a button), edit the entire block as one replacement rather than patching opening/closing tags separately. Deeply nested Svelte templates make it easy to introduce mismatched `</div>` tags when doing incremental edits.
