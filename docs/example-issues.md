# Example GitHub Issues for AgentFlow Tutorial

This file contains the full issue bodies used in the [TUTORIAL.md](../TUTORIAL.md) target project setup. Copy the `--body` content when creating issues via the GitHub Web UI, or use the `gh issue create` commands from the tutorial.

---

## Issue 1: Backend Authentication & Authorization

**Title:** `Implement backend authentication and authorization system`

**Body:**

```markdown
## Description

Build a complete JWT-based authentication system with role-based access control for the inventory management API.

## Requirements

### User Model (backend/src/models/User.js)
- Fields: email, password (hashed), role (admin/manager/viewer), firstName, lastName, isActive, lastLogin, createdAt, updatedAt
- Email validation and uniqueness constraint
- Password hashing with bcrypt (12 rounds)
- Instance methods: comparePassword(), generateToken(), hasPermission(permission)

### Auth Routes (backend/src/routes/auth.js)
- POST /api/auth/register - User registration with email verification token
- POST /api/auth/login - Login with JWT generation (access token: 15min, refresh token: 7d)
- POST /api/auth/refresh - Token refresh endpoint
- POST /api/auth/logout - Invalidate refresh token
- POST /api/auth/forgot-password - Send password reset email
- POST /api/auth/reset-password - Reset password with token
- GET /api/auth/verify-email/:token - Email verification

### Middleware (backend/src/middleware/auth.js)
- authenticate - Verify JWT access token
- authorize(...roles) - Check user role against allowed roles
- rateLimit - API rate limiting (100 req/15min for auth endpoints)

### Controllers (backend/src/controllers/authController.js)
- Handle all auth logic with proper error handling
- Return standardized error responses
- Log authentication attempts

### Tests (backend/tests/auth.test.js)
- Unit tests for User model methods
- Integration tests for all auth endpoints
- Test role-based authorization
- Test token expiration and refresh
- Test password reset flow

## Acceptance Criteria

- [ ] User can register with valid email and password
- [ ] Duplicate email registration is rejected
- [ ] Password is never stored in plaintext
- [ ] Login returns access and refresh tokens
- [ ] Expired access tokens are rejected
- [ ] Refresh token rotation works correctly
- [ ] Role-based middleware blocks unauthorized access
- [ ] Password reset flow works end-to-end
- [ ] All auth endpoints have rate limiting
- [ ] Test coverage >= 80%
- [ ] All endpoints return proper HTTP status codes
- [ ] Errors are logged with Winston

## API Response Format

Success:
```json
{
  "success": true,
  "data": { ... },
  "message": "Operation successful"
}
```

Error:
```json
{
  "success": false,
  "error": {
    "code": "AUTH_001",
    "message": "Invalid credentials",
    "details": []
  }
}
```

## Security Notes

- Use environment variables for JWT_SECRET, JWT_REFRESH_SECRET
- Implement token blacklisting for logout
- Add XSS protection headers
- Use HTTPS in production
- Sanitize all user inputs
```

---

## Issue 2: Inventory CRUD API

**Title:** `Build inventory CRUD API with stock management and audit logging`

**Body:**

```markdown
## Description

Implement the core inventory management API with full CRUD operations, stock level tracking, low-stock alerts, and comprehensive audit logging.

## Requirements

### Inventory Item Model (backend/src/models/InventoryItem.js)
- Fields: sku (unique), name, description, category, quantity, minStockLevel, maxStockLevel, unitPrice, supplier, location, status (active/discontinued), createdAt, updatedAt
- Virtual fields: stockValue (quantity * unitPrice), needsReorder (quantity <= minStockLevel)
- Indexes on: sku, category, status, supplier
- Pre-save hooks for SKU generation and timestamp updates

### Stock Transaction Model (backend/src/models/StockTransaction.js)
- Fields: itemId, type (in/out/adjustment), quantity, previousQuantity, newQuantity, reason, performedBy (user ref), notes, createdAt
- Immutable records - never update or delete transactions
- Aggregate queries for stock history

### Audit Log Model (backend/src/models/AuditLog.js)
- Fields: entityType, entityId, action, previousState, newState, performedBy, ipAddress, userAgent, timestamp
- Automatic logging via Mongoose middleware
- Retention policy: 90 days for detailed logs, 1 year for summary

### Inventory Routes (backend/src/routes/inventory.js)
- GET /api/inventory - List items with pagination, filtering, sorting
- GET /api/inventory/:id - Get single item with stock history
- POST /api/inventory - Create new item (manager+ only)
- PUT /api/inventory/:id - Update item (manager+ only)
- DELETE /api/inventory/:id - Soft delete item (admin only)
- POST /api/inventory/:id/stock - Record stock movement
- GET /api/inventory/low-stock - Get items below minStockLevel
- GET /api/inventory/categories - List unique categories
- GET /api/inventory/stats - Dashboard statistics

### Inventory Controllers (backend/src/controllers/inventoryController.js)
- Pagination: default 20 items/page, max 100
- Filtering: by category, status, supplier, stock level range
- Sorting: by name, quantity, stockValue, updatedAt
- Search: full-text search on name, description, SKU
- Stock adjustments: validate quantity changes, create transaction records
- Low-stock alerts: return items needing reorder with suggested quantities

### Tests (backend/tests/inventory.test.js)
- CRUD operations for inventory items
- Stock movement recording and validation
- Audit log creation on all mutations
- Pagination, filtering, and sorting
- Authorization checks on protected routes
- Edge cases: negative quantities, invalid SKUs, concurrent updates

## Acceptance Criteria

- [ ] All CRUD operations work correctly
- [ ] Stock movements create immutable transaction records
- [ ] Audit logs capture all create/update/delete operations
- [ ] Pagination returns correct page info (total, pages, current)
- [ ] Filtering by multiple criteria works with AND logic
- [ ] Search returns relevant results across name, description, SKU
- [ ] Low-stock endpoint returns accurate items
- [ ] Stats endpoint returns: total items, total value, low stock count, categories count
- [ ] Concurrent stock updates are handled safely
- [ ] Test coverage >= 80%
- [ ] All endpoints validate input with express-validator
- [ ] Error responses include actionable error codes

## Performance Requirements

- List endpoint: < 200ms for 10k items with pagination
- Single item: < 50ms
- Stock movement: < 100ms including audit log
- Stats endpoint: < 300ms with aggregation pipeline

## Database Indexes Required

- InventoryItem: { sku: 1 }, { category: 1, status: 1 }, { quantity: 1 }, { supplier: 1 }
- StockTransaction: { itemId: 1, createdAt: -1 }, { performedBy: 1 }
- AuditLog: { entityType: 1, entityId: 1 }, { createdAt: -1 }
```

---

## Issue 3: React Frontend

**Title:** `Build React frontend with dashboard, inventory management UI, and analytics`

**Body:**

```markdown
## Description

Create a complete React frontend application with Material-UI components, Redux state management, role-based UI rendering, inventory management interface, and analytics dashboard with charts.

## Requirements

### Project Structure
```
frontend/src/
├── components/
│   ├── layout/
│   │   ├── AppLayout.jsx       # Main layout with sidebar
│   │   ├── Sidebar.jsx         # Navigation sidebar
│   │   ├── Header.jsx          # Top header with user menu
│   │   └── Footer.jsx
│   ├── common/
│   │   ├── DataTable.jsx       # Reusable data table with pagination
│   │   ├── SearchBar.jsx       # Search with debounce
│   │   ├── StatusBadge.jsx     # Color-coded status indicators
│   │   ├── ConfirmDialog.jsx   # Confirmation modal
│   │   ├── LoadingSpinner.jsx
│   │   └── ErrorBoundary.jsx
│   └── inventory/
│       ├── ItemForm.jsx        # Create/edit item form
│       ├── ItemCard.jsx        # Single item display
│       ├── StockMovementForm.jsx
│       └── ItemTable.jsx
├── pages/
│   ├── LoginPage.jsx
│   ├── DashboardPage.jsx       # Main dashboard with stats
│   ├── InventoryPage.jsx       # Item list with CRUD
│   ├── ItemDetailPage.jsx      # Single item view
│   ├── AnalyticsPage.jsx       # Charts and reports
│   ├── UsersPage.jsx           # User management (admin)
│   └── SettingsPage.jsx
├── store/
│   ├── index.js                # Redux store configuration
│   ├── slices/
│   │   ├── authSlice.js        # Auth state and thunks
│   │   ├── inventorySlice.js   # Inventory state and thunks
│   │   ├── uiSlice.js          # UI state (loading, errors, modals)
│   │   └── analyticsSlice.js
│   └── middleware/
│       └── authMiddleware.js   # Token refresh interceptor
├── services/
│   ├── api.js                  # Axios instance with interceptors
│   ├── authService.js          # Auth API calls
│   ├── inventoryService.js     # Inventory API calls
│   └── analyticsService.js
├── utils/
│   ├── formatters.js           # Date, currency, number formatting
│   ├── validators.js           # Form validation helpers
│   └── constants.js            # App constants
├── hooks/
│   ├── useAuth.js              # Auth context hook
│   ├── useDebounce.js
│   └── usePermissions.js       # Role-based permission hook
├── routes/
│   ├── AppRoutes.jsx           # Main route configuration
│   ├── ProtectedRoute.jsx      # Auth guard
│   └── AdminRoute.jsx          # Admin-only guard
└── theme/
    └── index.js                # Material-UI theme configuration
```

### Key Features

1. **Authentication Flow**
   - Login page with email/password
   - Remember me with secure token storage
   - Password reset flow
   - Auto-redirect after login
   - Session timeout handling

2. **Dashboard**
   - Summary cards: Total items, Total value, Low stock count, Categories
   - Recent activity feed (last 10 audit log entries)
   - Low stock alert table
   - Quick actions: Add item, Export report, View analytics
   - Real-time updates via polling (every 30s)

3. **Inventory Management**
   - Data table with:
     - Pagination (client-side for < 1000, server-side for > 1000)
     - Column sorting (click headers)
     - Multi-filter (category, status, supplier, stock level)
     - Search with 300ms debounce
     - Row selection with bulk actions
     - Export to CSV
   - Item detail view with:
     - Full item information
     - Stock history timeline
     - Audit log for this item
     - Edit/Delete buttons (role-based)
   - Add/Edit item form with:
     - Form validation (react-hook-form + yup)
     - Real-time field validation
     - Auto-generate SKU button
     - Image upload (optional)
     - Draft saving

4. **Analytics Page**
   - Stock level trends (line chart)
   - Category distribution (pie chart)
   - Top suppliers by value (bar chart)
   - Stock movement summary (table)
   - Date range picker for all charts
   - Export charts as PNG

5. **Role-Based UI**
   - Admin: Full access, user management, system settings
   - Manager: Inventory CRUD, stock movements, analytics
   - Viewer: Read-only access to inventory and analytics

### State Management

- Redux Toolkit for global state
- RTK Query or custom thunks for API calls
- Local state for form inputs and UI toggles
- Persist auth state to localStorage (encrypted)

### Testing (frontend/tests/)

- Unit tests for all utility functions
- Component tests for common components
- Integration tests for auth flow
- Mock API responses with MSW
- Test role-based UI rendering
- Test form validation

## Acceptance Criteria

- [ ] Login/logout flow works with token management
- [ ] Dashboard displays accurate summary statistics
- [ ] Inventory table supports pagination, sorting, filtering, search
- [ ] Add/Edit form validates all fields correctly
- [ ] Stock movement form creates proper API requests
- [ ] Analytics page renders all charts correctly
- [ ] Role-based UI shows/hides appropriate elements
- [ ] All API errors are displayed to user gracefully
- [ ] Loading states shown during async operations
- [ ] Responsive design works on mobile, tablet, desktop
- [ ] Test coverage >= 70%
- [ ] No console errors or warnings
- [ ] Accessibility: All interactive elements keyboard-navigable
- [ ] Performance: Initial load < 3s, page transitions < 500ms

## Design Guidelines

- Material-UI v5 as component library
- Color scheme: Primary #1976d2, Secondary #424242, Success #2e7d32, Warning #ed6c02, Error #d32f2f
- Typography: Roboto font family
- Spacing: 8px base unit
- Shadows: Use MUI elevation system
- Icons: Material Icons
- Dark mode support (toggle in settings)
```
