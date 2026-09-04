# Responsive Terminal Layout

flareTUI features a responsive modal engine (`centered_rect`) ensuring optimal display across all viewport sizes.

## Adaptive Rules

- **Standard Viewports (80x24 up to 120x40)**: Centered modals with proportional width (60-80%) and automatic line wrapping.
- **Compact Viewports (< 80 columns)**: Clamped margins maximizing visible content and preventing truncation.
- **Ultrawide Displays (> 160 columns)**: Fixed maximum constraints preventing awkward horizontal stretching.
