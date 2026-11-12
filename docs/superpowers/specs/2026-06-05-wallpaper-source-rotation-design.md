# Wallpaper Source Rotation Design

## Goal

Adjust the source lottery so that starting from the third overall draw, wallpaper prizes follow a repeating quota of four wallpaper outcomes: exactly three API wallpapers and one server-gallery wallpaper, with randomized order inside each four-item batch.

## Scope

- Only applies when the drawn prize type is `wallpaper`.
- The first overall draw stays fixed to `homepage`.
- The second overall draw stays fixed to `blog`.
- Starting with the third overall draw, prize selection still uses the existing probability pool.
- When a wallpaper prize is selected, the wallpaper source is chosen from a shuffled four-item source queue containing:
  - `api`
  - `api`
  - `api`
  - `gallery`
- Once the queue is exhausted, generate a new shuffled four-item queue and continue.
- Hidden test draw `999` continues to force API-only behavior and does not consume the normal wallpaper source queue.

## Implementation Shape

- Keep the rule in the frontend session state inside `main/src/components/Contact.jsx`.
- Add a helper that creates a shuffled wallpaper source queue for one batch.
- Add state for the remaining queue items.
- On each normal wallpaper prize draw, pop one source from the queue.
- If the source is `api`, call the existing backend wallpaper API.
- If the source is `gallery`, use the existing local gallery picker.
- Update the left-side manual text to describe the new 3:1 wallpaper-source rule in Chinese.

## Non-Goals

- No backend quota tracking.
- No persistence across page refreshes.
- No changes to the overall prize probability split.
- No changes to the special `999` API test path beyond keeping it excluded from the normal queue.

## Validation

- Frontend build must pass.
- Targeted lint for `Contact.jsx` and related service code must pass or only show pre-existing warnings.
- Manual behavior expectation:
  - Ignore the first two fixed non-wallpaper draws.
  - For any four normal wallpaper results after that, the set must contain exactly three API results and one gallery result.
  - The order within a batch must vary.
