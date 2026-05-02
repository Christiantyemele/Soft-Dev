#!/bin/bash
# Initialize target project structure for AgentFlow tutorial
# Run this inside your newly cloned enterprise-inventory repository

set -e

echo "Creating project structure..."

# Create directories
mkdir -p backend/src/{models,routes,middleware,controllers,utils}
mkdir -p backend/tests
mkdir -p frontend/src/{components,pages,services,store,utils}
mkdir -p frontend/public
mkdir -p frontend/tests
mkdir -p .github/workflows

# Create backend package.json
cat > backend/package.json << 'EOF'
{
  "name": "inventory-backend",
  "version": "1.0.0",
  "description": "Enterprise Inventory Management API",
  "main": "src/server.js",
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint src/",
    "seed": "node src/utils/seed.js"
  },
  "dependencies": {
    "express": "^4.18.0",
    "mongoose": "^8.0.0",
    "jsonwebtoken": "^9.0.0",
    "bcryptjs": "^2.4.3",
    "cors": "^2.8.5",
    "dotenv": "^16.0.0",
    "express-validator": "^7.0.0",
    "winston": "^3.11.0"
  },
  "devDependencies": {
    "jest": "^29.7.0",
    "nodemon": "^3.0.0",
    "eslint": "^8.56.0",
    "supertest": "^6.3.0"
  }
}
EOF

# Create frontend package.json
cat > frontend/package.json << 'EOF'
{
  "name": "inventory-frontend",
  "version": "1.0.0",
  "private": true,
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.21.0",
    "@reduxjs/toolkit": "^2.0.0",
    "react-redux": "^9.0.0",
    "axios": "^1.6.0",
    "@mui/material": "^5.15.0",
    "@emotion/react": "^11.11.0",
    "@emotion/styled": "^11.11.0",
    "recharts": "^2.10.0",
    "react-hook-form": "^7.49.0",
    "date-fns": "^3.0.0"
  },
  "devDependencies": {
    "@testing-library/react": "^14.1.0",
    "@testing-library/jest-dom": "^6.2.0",
    "vitest": "^1.1.0",
    "vite": "^5.0.0"
  },
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest run",
    "lint": "eslint src/"
  }
}
EOF

# Create initial README
cat > README.md << 'EOF'
# Enterprise Inventory Management System

A fullstack inventory management application with role-based access control, real-time stock tracking, analytics dashboard, and automated reorder alerts.

## Architecture

- **Backend**: Node.js/Express REST API with MongoDB
- **Frontend**: React 18 with Redux Toolkit, Material-UI, and Recharts
- **Auth**: JWT-based with role-based access control (Admin, Manager, Viewer)
- **Testing**: Jest for backend, Vitest + React Testing Library for frontend

## Features

- User authentication and authorization
- CRUD operations for inventory items
- Stock level tracking with automated reorder alerts
- Analytics dashboard with charts
- Audit logging for all inventory changes
- Export reports (CSV, PDF)
EOF

# Commit and push
git add .
git commit -m "Initial project structure for enterprise inventory system"
git push origin main

echo "Project structure created and pushed successfully!"
echo "Next: Create GitHub issues (see docs/example-issues.md in AgentFlow repo)"
