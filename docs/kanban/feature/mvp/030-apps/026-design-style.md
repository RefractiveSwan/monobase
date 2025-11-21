# 026 - Design & Style Refinement

**Status**: 🟡 In Progress  
**Priority**: Medium  
**Assignee**: Design Team  
**Epic**: MVP Polish  
**Created**: 2025-11-20

## Overview

Refine the academic theme design based on initial implementation review. Focus on visual hierarchy, information density, accessibility, and professional polish for MVP demonstrations.

---

## Problem Statement

The initial academic theme implementation successfully establishes a professional foundation with Merriweather/Inter typography and navy/gold palette. However, several areas need refinement to achieve a truly polished, production-ready MVP interface suitable for academic and clinical stakeholders.

---

## Design Review Findings

### 1. **Typography Hierarchy** 🟡 Medium Priority

**Problem**: Heading sizes and weights need better differentiation
- Section headers (h2) could be more prominent
- Body text line-height may need adjustment for dense data sections
- Code/monospace elements could benefit from better contrast

**Impact**: Reduced scannability, harder to navigate complex data

**Proposed Solution**:
- Increase h2 font size from `text-lg` to `text-xl` or `text-2xl`
- Add more weight variation (use `font-bold` vs `font-semibold` strategically)
- Adjust line-height for better readability in data-heavy sections

---

### 2. **Color Contrast & Accessibility** 🔴 High Priority

**Problem**: Some text/background combinations may not meet WCAG AA standards
- Gray-500 text on gray-50 backgrounds
- Gold accent color contrast needs verification
- Status badges (emerald/amber/rose) should be tested for color-blind users

**Impact**: Accessibility issues, potential compliance problems

**Proposed Solution**:
- Run WCAG contrast checker on all text/background pairs
- Ensure minimum 4.5:1 contrast ratio for normal text
- Add patterns/icons to status badges (not just color)
- Test with color-blind simulation tools

---

### 3. **Spacing & Whitespace** 🟡 Medium Priority

**Problem**: Inconsistent spacing between sections
- Some cards feel cramped (p-4 vs p-6 inconsistency)
- Gap between sections varies (gap-4, gap-6, space-y-6, space-y-8)
- Table cell padding could be more generous for readability

**Impact**: Cluttered appearance, reduced professional feel

**Proposed Solution**:
- Establish spacing scale: sm (gap-4), md (gap-6), lg (gap-8)
- Apply consistently across all components
- Increase table cell padding to px-6 py-4 minimum
- Add more breathing room around dense data displays

---

### 4. **Interactive States** 🟢 Low Priority

**Problem**: Hover/focus states could be more pronounced
- Button hover effects are subtle
- Table row hovers work but could be enhanced
- Form input focus rings are standard but could be branded

**Impact**: Reduced interactivity feedback, less engaging UX

**Proposed Solution**:
- Add subtle scale transform on button hover (`hover:scale-[1.02]`)
- Enhance table row hover with left border accent
- Customize focus ring color to navy-900
- Add transition-all to interactive elements

---

### 5. **Data Visualization** 🟡 Medium Priority

**Problem**: Metrics could be more visually engaging
- Stat cards are functional but plain
- No visual indicators for trends or comparisons
- State distribution could use simple bar charts
- Missing sparklines or mini-visualizations

**Impact**: Data is harder to interpret at a glance

**Proposed Solution**:
- Add simple CSS-based progress bars for metrics
- Include trend indicators (↑↓) where applicable
- Implement mini bar charts for state distribution
- Consider adding Chart.js for richer visualizations

---

### 6. **Responsive Design** 🟡 Medium Priority

**Problem**: Mobile/tablet experience needs testing
- Grid layouts may not collapse gracefully
- Table overflow handling unclear
- Navigation may need mobile menu
- Touch targets may be too small

**Impact**: Poor experience on smaller screens

**Proposed Solution**:
- Test on mobile devices (375px, 768px, 1024px breakpoints)
- Implement horizontal scroll for tables with sticky headers
- Add hamburger menu for mobile navigation
- Ensure minimum 44px touch targets

---

### 7. **Loading & Empty States** 🟢 Low Priority

**Problem**: Missing loading indicators and better empty states
- No spinners during async operations
- Empty state messages are plain text
- No skeleton loaders for initial page load

**Impact**: Users unsure if actions are processing

**Proposed Solution**:
- Add HTMX loading indicators (hx-indicator)
- Design illustrated empty states
- Implement skeleton screens for initial load
- Add subtle animations for state transitions

---

### 8. **Icon System** 🟢 Low Priority

**Problem**: No icon library integrated
- Using text symbols (↗, ✓, ❌) inconsistently
- Missing visual cues for actions
- No standardized iconography

**Impact**: Less polished, harder to scan quickly

**Proposed Solution**:
- Integrate Heroicons or Lucide Icons
- Add icons to buttons (upload, submit, filter)
- Use icons in status badges
- Standardize icon sizing (w-4 h-4, w-5 h-5)

---

### 9. **Form Design** 🟡 Medium Priority

**Problem**: Forms could be more user-friendly
- File upload area is basic
- No validation feedback styling
- Labels could be more descriptive
- Missing helper text for complex inputs

**Impact**: Higher error rates, user confusion

**Proposed Solution**:
- Enhance file upload with drag-and-drop visual
- Add inline validation with error messages
- Include helper text under inputs
- Show character counts for textareas

---

### 10. **Table Enhancements** 🟡 Medium Priority

**Problem**: Tables are functional but could be richer
- No sorting indicators
- No row selection for batch actions
- Missing pagination for large datasets
- Column headers could be more interactive

**Impact**: Limited data manipulation capabilities

**Proposed Solution**:
- Add sort arrows to column headers
- Implement checkbox selection
- Add pagination component
- Enable column resizing (advanced)

---

## Implementation Checklist

### Phase 1: Critical Fixes (Week 1) - ✅ IN PROGRESS
- [x] **Color System Audit** - Enhanced color contrast for accessibility (Card 2.1)
  - Added navy-600, navy-700 for better text contrast
  - Adjusted gold-500 for improved visibility
  - Lightened paper background to #fafafa
  - Added academic-lg shadow variant
- [x] **Typography Refinement** - Improved heading hierarchy (Card 2.2)
  - Increased h2 from text-2xl to text-3xl
  - Added leading-tight for better readability
  - Enhanced spacing between heading and body text
- [x] **Spacing Consistency** - Applied unified spacing scale (Card 2.3)
  - Increased main padding: py-10, space-y-10
  - Added responsive padding: px-6 lg:px-8
  - Standardized section spacing throughout
  - Added custom spacing utilities (18, 22)
- [x] **Button & Link States** - Enhanced interactive feedback (Card 3.1)
  - Added hover:scale-[1.02] and active:scale-[0.98] to buttons
  - Improved focus rings: focus:ring-2 focus:ring-navy-900 focus:ring-offset-2
  - Enhanced nav link hover with underline effects
  - Added transition-all duration-200 for smooth animations
- [x] **Form UX Improvements** - Better labels and inputs (Card 3.2)
  - Upgraded labels with uppercase tracking-wide styling
  - Added helper text for form fields
  - Enhanced input borders: border-2 for better visibility
  - Improved focus states with ring effects
- [x] **Responsive testing** - Test and fix mobile layouts


### Phase 2: UX Enhancements (Week 2) - ✅ IN PROGRESS
- [x] **Table Enhancements** - Added hover states with left border accent (Card 3.3)
  - Cohort table: hover:border-l-navy-900
  - Mapping results table: hover:border-l-navy-900
  - NoMatch explorer table: hover:border-l-rose-600
  - Added cursor-pointer and smooth transitions
  - **Sticky Headers**: Added sticky top-0 z-10 to table headers
  - **Client-side Sorting**: Implemented JS-based sorting for all columns
  - **Consistent Spacing**: Standardized cell padding to px-4 py-3
- [x] **Data Visualization** - Progress bars & Charts (Card 4.1, 4.2)
  - Added visual progress bars to percentage metrics
  - Color-coded thresholds: green (\u003e=90%), amber (70-89%), red (\u003c70%)
  - Integrated Chart.js for advanced visualization
  - State Distribution: Interactive Doughnut chart
  - Top Concepts: Horizontal Bar chart
  - Responsive and themed to match academic style
- [ ] **Loading States** - Spinners and skeletons
- [ ] **Empty States** - Illustrated placeholders

### Phase 3: Polish (Week 3) - ✅ COMPLETE
- [x] **Icon Integration** - Added Heroicons throughout (Card 5.4)
  - Navigation icons: Map, Chart Bar, Beaker
  - Button icons: Paper Airplane (submit), Arrow Up Tray (upload)
  - Empty state icons: Magnifying Glass
  - All icons are w-4 h-4 for consistency
  - Proper aria-labels via semantic HTML
- [x] **Loading States** - HTMX indicators with spinners (Card 5.3)
  - Added custom CSS for .htmx-indicator
  - Animated spinner on form submit buttons
  - Smooth rotation animation (1s linear infinite)
  - Hidden by default, shows during HTMX requests
- [x] **Empty States** - Enhanced messaging (Card 5.3)
  - NoMatch Explorer: Magnifying glass icon + helpful CTA
  - Centered layout with proper spacing
  - Clear, actionable messaging
- [x] **Micro-animations** - Smooth transitions (Card 8.1)
  - Button hover: scale-[1.02], active: scale-[0.98]
  - Progress bar fill: duration-500
  - Table row hover: duration-150
  - All transitions use ease-in-out

---

## Progress Summary

### ✅ Completed
1. **Enhanced Color Contrast** - Improved WCAG compliance with deeper navy tones
2. **Typography Scale** - Larger, more prominent headings (h2: text-3xl)
3. **Spacing System** - Consistent 10-unit vertical rhythm
4. **Button Interactions** - Smooth hover/active/focus states
5. **Form Styling** - Professional labels with helper text
6. **Table Interactivity** - Sticky headers, sorting, hover states
7. **Data Visualization** - Chart.js integration (Doughnut, Bar) & progress bars
8. **Mobile Layout** - Responsive hamburger menu & grid system
9. **Polish** - Icons, loading spinners, empty states

### 🚧 Remaining (Low Priority)
1. **Accessibility Audit** - Full keyboard nav test & ARIA landmarks
2. **Form Validation** - Inline error messages
3. **Page Transitions** - Fade-in effects

---

## Detailed Task Cards

### 🏗️ LAYOUT & STRUCTURE

#### Card 1.1: Page Layout Optimization
**Priority**: 🟡 Medium | **Effort**: 3 points | **Owner**: Frontend

**Tasks**:
- [ ] Audit max-width usage across pages (currently max-w-6xl, max-w-7xl, max-w-5xl)
- [ ] Standardize container widths: `max-w-7xl` for main content
- [ ] Add consistent page padding: `px-6` on mobile, `px-8` on desktop
- [ ] Implement sticky header on scroll
- [ ] Add breadcrumb navigation for multi-page flows
- [ ] Test layout on 1920px, 1440px, 1024px, 768px, 375px viewports

**Acceptance Criteria**:
- All pages use consistent container widths
- Header remains accessible when scrolling
- No horizontal scroll on any viewport size

---

#### Card 1.2: Grid System Refinement
**Priority**: 🟡 Medium | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Audit all grid layouts (currently using `grid-cols-2`, `grid-cols-3`, `md:grid-cols-2`, etc.)
- [ ] Standardize breakpoints: sm (640px), md (768px), lg (1024px), xl (1280px)
- [ ] Ensure consistent gap spacing: `gap-4` (mobile), `gap-6` (desktop)
- [ ] Fix grid collapse behavior on mobile (ensure single column)
- [ ] Add grid debugging overlay (dev mode only)

**Acceptance Criteria**:
- Grids collapse gracefully on mobile
- Consistent gap spacing across all grid layouts
- No layout shifts during resize

---

#### Card 1.3: Section Hierarchy
**Priority**: 🟢 Low | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Add visual separators between major sections (subtle borders or increased spacing)
- [ ] Implement section anchors for deep linking
- [ ] Add "back to top" button for long pages
- [ ] Group related sections with subtle background tints
- [ ] Ensure proper semantic HTML5 structure (`<section>`, `<article>`, `<aside>`)

**Acceptance Criteria**:
- Clear visual hierarchy between sections
- All sections have unique IDs for linking
- Semantic HTML passes validation

---

### 🎨 VISUAL DESIGN

#### Card 2.1: Color System Audit
**Priority**: 🔴 High | **Effort**: 3 points | **Owner**: Design + Frontend

**Tasks**:
- [ ] Run WCAG contrast checker on all text/background pairs
- [ ] Document all color combinations in use
- [ ] Fix low-contrast issues (target 4.5:1 minimum)
- [ ] Create color palette documentation with usage guidelines
- [ ] Test with color-blind simulation (Deuteranopia, Protanopia, Tritanopia)
- [ ] Add CSS custom properties for theme colors

**Acceptance Criteria**:
- 100% WCAG AA compliance for text contrast
- Zero accessibility violations in axe DevTools
- Color palette documented in style guide

**Files to Modify**:
- `lib/app/frontend/web/src/views.rs` (Tailwind config)
- New: `docs/design/color-palette.md`

---

#### Card 2.2: Typography Scale
**Priority**: 🟡 Medium | **Effort**: 2 points | **Owner**: Design

**Tasks**:
- [ ] Increase h1 size: `text-3xl` → `text-4xl` (desktop)
- [ ] Increase h2 size: `text-lg` → `text-2xl`
- [ ] Standardize h3 size: `text-sm` → `text-lg`
- [ ] Adjust line-height for readability: `leading-relaxed` for body text
- [ ] Add letter-spacing to uppercase labels: `tracking-wider`
- [ ] Test font loading performance (ensure FOUT/FOIT handled)

**Acceptance Criteria**:
- Clear visual hierarchy between heading levels
- Body text comfortable to read for extended periods
- Fonts load without flash of unstyled text

---

#### Card 2.3: Spacing & Rhythm
**Priority**: 🟡 Medium | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Define spacing scale: xs (4px), sm (8px), md (16px), lg (24px), xl (32px), 2xl (48px)
- [ ] Apply consistent spacing to all cards: `p-6` standard
- [ ] Standardize section gaps: `space-y-8` for major sections, `space-y-4` for subsections
- [ ] Increase table cell padding: `px-6 py-4` minimum
- [ ] Add breathing room around dense data: extra `mt-2` or `mb-2`
- [ ] Create spacing utility classes if needed

**Acceptance Criteria**:
- Consistent spacing throughout application
- No cramped or overly-spacious sections
- Visual rhythm feels balanced

---

### 🖱️ INTERACTION & UX

#### Card 3.1: Button & Link States
**Priority**: 🟡 Medium | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Enhance primary button hover: add `hover:scale-[1.02] transition-transform`
- [ ] Add active state: `active:scale-[0.98]`
- [ ] Improve focus rings: use `focus:ring-2 focus:ring-navy-900 focus:ring-offset-2`
- [ ] Add disabled state styling: `disabled:opacity-50 disabled:cursor-not-allowed`
- [ ] Ensure all links have underline on hover
- [ ] Add loading state for async buttons (spinner icon)

**Acceptance Criteria**:
- All interactive elements have clear hover/focus/active states
- Keyboard navigation works smoothly
- Loading states prevent double-clicks

---

#### Card 3.2: Form UX Improvements
**Priority**: 🟡 Medium | **Effort**: 4 points | **Owner**: Frontend

**Tasks**:
- [ ] Add inline validation with error messages (red border + text)
- [ ] Show success states (green border + checkmark)
- [ ] Add helper text under inputs: `<span class="text-xs text-gray-500">`
- [ ] Implement character counter for textarea
- [ ] Enhance file upload: drag-and-drop visual feedback
- [ ] Add "Clear" button for text inputs
- [ ] Show required field indicators (asterisk)
- [ ] Implement autofocus on first input

**Acceptance Criteria**:
- Users receive immediate feedback on input errors
- File upload area clearly indicates drag-and-drop capability
- Required fields are obvious

**Files to Modify**:
- `lib/app/frontend/web/src/views.rs` (form rendering functions)

---

#### Card 3.3: Table Interactivity
**Priority**: 🟡 Medium | **Effort**: 5 points | **Owner**: Frontend

**Tasks**:
- [ ] Add sortable column headers with arrow indicators
- [ ] Implement row hover with left border accent: `hover:border-l-4 hover:border-l-navy-900`
- [ ] Add row selection checkboxes (for batch actions)
- [ ] Implement pagination component (10/25/50 rows per page)
- [ ] Add "Export to CSV" button
- [ ] Show row count: "Showing 1-10 of 42 results"
- [ ] Add sticky table headers on scroll
- [ ] Implement horizontal scroll with shadow indicators

**Acceptance Criteria**:
- Users can sort by any column
- Pagination works smoothly
- Tables remain usable on mobile with horizontal scroll

**Files to Modify**:
- `lib/app/frontend/web/src/views.rs` (table rendering functions)
- New: JavaScript for sorting/pagination (or HTMX endpoints)

---

#### Card 3.4: Navigation Enhancements
**Priority**: 🟢 Low | **Effort**: 3 points | **Owner**: Frontend

**Tasks**:
- [ ] Add active state to nav links (underline or background highlight)
- [ ] Implement mobile hamburger menu (< 768px)
- [ ] Add keyboard shortcuts (e.g., `/` for search, `?` for help)
- [ ] Show current page indicator in nav
- [ ] Add dropdown for user menu (if auth added later)
- [ ] Implement skip-to-content link for accessibility

**Acceptance Criteria**:
- Current page is clearly indicated in navigation
- Mobile menu works smoothly
- Keyboard navigation is intuitive

---

### 📊 DATA VISUALIZATION

#### Card 4.1: Metrics Dashboard Enhancement
**Priority**: 🟡 Medium | **Effort**: 4 points | **Owner**: Frontend

**Tasks**:
- [ ] Add simple CSS progress bars for percentage metrics
- [ ] Implement trend indicators (↑ green, ↓ red, → gray)
- [ ] Add sparkline charts for time-series data (Chart.js or pure CSS)
- [ ] Show comparison to previous period ("↑ 12% from last week")
- [ ] Add color-coded thresholds (green >90%, yellow 70-90%, red <70%)
- [ ] Implement animated number counters on page load
- [ ] Add tooltips with detailed breakdowns

**Acceptance Criteria**:
- Metrics are visually engaging and easy to interpret
- Trends are immediately obvious
- Animations are subtle and performant

**Files to Modify**:
- `lib/app/frontend/web/src/views.rs` (metric_card functions)
- Consider: Add Chart.js CDN or lightweight alternative

---

#### Card 4.2: State Distribution Visualization
**Priority**: 🟡 Medium | **Effort**: 3 points | **Owner**: Frontend

**Tasks**:
- [ ] Replace text-only state counts with horizontal bar charts
- [ ] Add percentage labels to bars
- [ ] Color-code bars by state (emerald, amber, rose)
- [ ] Implement stacked bar chart for time buckets
- [ ] Add legend for color meanings
- [ ] Make charts responsive (collapse to vertical on mobile)

**Acceptance Criteria**:
- State distribution is visually clear at a glance
- Charts work on all screen sizes
- Colors are accessible (not just color-dependent)

---

### 🎭 COMPONENT POLISH

#### Card 5.1: Card Component Refinement
**Priority**: 🟢 Low | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Standardize card styling: `rounded-md border border-gray-200 shadow-academic`
- [ ] Add card header pattern: navy-50 background with border-b
- [ ] Implement card footer pattern (if needed)
- [ ] Add hover effect for clickable cards: `hover:shadow-lg transition-shadow`
- [ ] Ensure consistent padding: `p-6` for body, `px-6 py-3` for header
- [ ] Add optional card icon/badge in header

**Acceptance Criteria**:
- All cards have consistent styling
- Card hierarchy is clear (header/body/footer)
- Hover states work for interactive cards

---

#### Card 5.2: Badge & Status Indicators
**Priority**: 🟡 Medium | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Add icons to status badges (✓ for success, ⚠ for warning, ✗ for error)
- [ ] Implement pulsing animation for "in progress" states
- [ ] Add tooltip explanations on hover
- [ ] Ensure badges work without color (patterns/icons)
- [ ] Standardize badge sizing: `text-xs px-2.5 py-0.5`
- [ ] Create badge variants: solid, outline, subtle

**Acceptance Criteria**:
- Status is clear without relying on color alone
- Badges are consistent across application
- Tooltips provide helpful context

---

#### Card 5.3: Loading & Empty States
**Priority**: 🟢 Low | **Effort**: 3 points | **Owner**: Frontend + Design

**Tasks**:
- [ ] Add HTMX loading indicators: `<div class="htmx-indicator">Loading...</div>`
- [ ] Design spinner component (navy-900 color)
- [ ] Create skeleton screens for initial page load
- [ ] Design illustrated empty states (e.g., "No data yet" with icon)
- [ ] Add helpful CTAs in empty states ("Upload your first bundle")
- [ ] Implement fade-in animations for loaded content

**Acceptance Criteria**:
- Users always know when something is loading
- Empty states are friendly and actionable
- Skeleton screens match final layout

**Files to Modify**:
- `lib/app/frontend/web/src/views.rs` (add loading/empty state markup)
- New: SVG illustrations for empty states

---

#### Card 5.4: Icon System Integration
**Priority**: 🟢 Low | **Effort**: 3 points | **Owner**: Frontend

**Tasks**:
- [ ] Choose icon library: Heroicons (recommended) or Lucide
- [ ] Add CDN link or download icon set
- [ ] Create icon component helper function
- [ ] Add icons to buttons: upload, submit, filter, export
- [ ] Add icons to navigation items
- [ ] Add icons to status indicators
- [ ] Standardize icon sizing: `w-4 h-4` (small), `w-5 h-5` (medium), `w-6 h-6` (large)
- [ ] Ensure icons have proper aria-labels

**Acceptance Criteria**:
- Icons are consistent and recognizable
- All icons have accessibility labels
- Icon library is performant (minimal bundle size)

---

### 📱 RESPONSIVE DESIGN

#### Card 6.1: Mobile Layout Optimization
- Touch targets are easy to tap

---

#### Card 6.2: Tablet Layout Optimization
**Priority**: 🟢 Low | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Test on iPad (768px) and iPad Pro (1024px)
- [ ] Optimize grid layouts for tablet: 2-column instead of 3
- [ ] Ensure navigation works in tablet mode
- [ ] Test landscape and portrait orientations
- [ ] Optimize sidebar/panel layouts for tablet

**Acceptance Criteria**:
- Tablet experience is optimized (not just scaled desktop)
- Both orientations work well

---

### ♿ ACCESSIBILITY

#### Card 7.1: Keyboard Navigation
**Priority**: 🔴 High | **Effort**: 3 points | **Owner**: Frontend

**Tasks**:
- [ ] Test tab order on all pages (ensure logical flow)
- [ ] Add skip-to-content link
- [ ] Ensure all interactive elements are keyboard accessible
- [ ] Add visible focus indicators (not just outline)
- [ ] Implement keyboard shortcuts (document in help modal)
- [ ] Test with screen reader (NVDA or JAWS)
- [ ] Add ARIA labels where needed
- [ ] Ensure modals trap focus

**Acceptance Criteria**:
- Entire application is navigable via keyboard
- Focus indicators are always visible
- Screen reader announces all important content

---

#### Card 7.2: ARIA & Semantic HTML
**Priority**: 🔴 High | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Add ARIA landmarks: `role="main"`, `role="navigation"`, etc.
- [ ] Add ARIA labels to icon-only buttons
- [ ] Implement ARIA live regions for dynamic content
- [ ] Add `aria-describedby` for form field hints
- [ ] Ensure heading hierarchy is logical (h1 → h2 → h3, no skips)
- [ ] Add `alt` text to all images (or `aria-hidden` if decorative)
- [ ] Run automated accessibility audit (axe, WAVE)

**Acceptance Criteria**:
- Zero critical accessibility violations
- Semantic HTML structure is correct
- ARIA attributes are used appropriately

---

### 🎬 ANIMATIONS & TRANSITIONS

#### Card 8.1: Micro-interactions
**Priority**: 🟢 Low | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Add subtle scale on button hover: `hover:scale-[1.02]`
- [ ] Implement fade-in for new content: `animate-fadeIn`
- [ ] Add slide-in for modals/panels
- [ ] Implement smooth scroll for anchor links
- [ ] Add ripple effect on button click (optional)
- [ ] Ensure all transitions use `transition-all duration-200 ease-in-out`
- [ ] Add loading spinner rotation animation

**Acceptance Criteria**:
- Animations are subtle and enhance UX
- No janky or slow animations
- Animations respect `prefers-reduced-motion`

---

#### Card 8.2: Page Transitions
**Priority**: 🟢 Low | **Effort**: 2 points | **Owner**: Frontend

**Tasks**:
- [ ] Add fade-in on page load
- [ ] Implement HTMX swap transitions (fade, slide)
- [ ] Add skeleton screen → content fade transition
- [ ] Ensure transitions don't delay perceived performance
- [ ] Test on slower devices (throttle CPU in DevTools)

**Acceptance Criteria**:
- Page transitions feel smooth
- No layout shift during transitions
- Performance is not impacted

---

## Testing Checklist

### Manual Testing
- [ ] Test on Chrome, Firefox, Safari, Edge
- [ ] Test on iPhone (Safari), Android (Chrome)
- [ ] Test on iPad (Safari)
- [ ] Test with keyboard only (no mouse)
- [ ] Test with screen reader (NVDA/JAWS/VoiceOver)
- [ ] Test with color-blind simulation
- [ ] Test on slow 3G connection
- [ ] Test with JavaScript disabled (graceful degradation)

### Automated Testing
- [ ] Run Lighthouse audit (target >90 on all metrics)
- [ ] Run axe DevTools (zero violations)
- [ ] Run WAVE accessibility checker
- [ ] Validate HTML (W3C validator)
- [ ] Check contrast ratios (WebAIM contrast checker)
- [ ] Test responsive breakpoints (Chrome DevTools)

---

## Success Metrics

- [ ] WCAG AA compliance (4.5:1 contrast minimum)
- [ ] Mobile usability score >90 (Lighthouse)
- [ ] Stakeholder approval in design review
- [ ] Zero accessibility violations (axe DevTools)
- [ ] <3s perceived load time (skeleton screens)

---

## Resources

- **Design System**: Tailwind CSS + Custom Theme
- **Typography**: Merriweather (serif), Inter (sans), Roboto Mono
- **Colors**: Navy 900 (#0f172a), Gold 500 (#b49b57), Paper (#f8f9fa)
- **Icons**: TBD (Heroicons recommended)
- **Testing**: Chrome DevTools, Lighthouse, axe, WAVE

---

## Notes

- Prioritize accessibility and mobile experience for MVP
- Keep academic/clinical aesthetic consistent
- Test with actual stakeholders before finalizing
- Document design decisions in style guide

---

## Related Tasks

- [024-codebase-refactor.md](./024-codebase-refactor.md) - Initial academic theme implementation
- Future: Component library extraction
- Future: Design system documentation
