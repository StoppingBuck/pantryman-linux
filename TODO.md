# TODO — Pantryman (Linux)

## Near term

- [ ] Desktop sync (mirror to a local folder that syncs via rclone/pCloud desktop)
- [ ] Recipe edit UI (currently read-only on desktop)
- [ ] Keyboard navigation improvements (pantry/ingredient list)
- [ ] Search/filter bar for ingredient and recipe lists

## Nice to have

- [ ] Dark mode / system theme following
- [ ] Print recipe view
- [ ] Shopping list generation (missing pantry items for a recipe set)
- [ ] Drag-and-drop reordering of pantry items

## Known issues

- Very long file paths in settings subtitle can overflow even with `set_subtitle_lines(1)` on some GTK themes
- DataManager reload after data dir change requires navigating away and back to see updated data in some edge cases
