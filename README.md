# nosDesk

**The Helpdesk Reimagined for Speed and Efficiency in Education**

`NosDesk` is a helpdesk solution designed specifically for educational institutions. It streamlines ticket management, enhances communication between staff and students, and optimizes workflows to ensure rapid resolution of issues in a school or university setting. Built with Vue.js, Tailwind CSS and other modern web technologies, `NosDesk` prioritizes speed, scalability, and user experience.

## Table of Contents
- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Development](#development)
- [Type Support](#type-support)
- [License](#license)

## Features
- **Education-Focused**: Tailored for schools, colleges, and universities to manage IT, facilities, and administrative issues.
- **Fast Ticket Resolution**: Intuitive interface and efficient workflows for quick issue tracking and resolution.
- **User Roles**: Supports roles like students, teachers, IT staff, facilities management, and administrators with role-based access control.
- **Real-Time Updates**: Live status updates for tickets and device management.
- **Customizable**: Easily configurable to fit the needs of different educational institutions.
- **Responsive Design**: Works seamlessly on desktops, tablets, and mobile devices.

## Getting Started

### Prerequisites
Before you begin, ensure you have the following installed:
- **Node.js** (v16 or later recommended) - [Download](https://nodejs.org/)
- **Git** (for cloning the repository) - [Download](https://git-scm.com/)

### Installation
1. Clone the repository:
   ```sh
   git clone https://github.com/your-username/nosDesk.git
   cd nosDesk
   ```
2. Install dependencies:
   ```sh
   npm install
   ```

## Development

To compile and hot-reload the application for development:
```sh
npm run dev
```
This launches the development server, typically accessible at `http://localhost:5173` (or the port specified by Vite).

## Type Support
`nosDesk` uses TypeScript for robust type checking. By default, TypeScript does not handle type information for `.vue` imports. To enable full type support:
- Use **[Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar)** in your IDE for `.vue` type awareness.
- Replace the standard `tsc` CLI with `vue-tsc` for type checking in your terminal:
  ```sh
  npx vue-tsc --noEmit
  ```

## License
`nosDesk` is licensed under the [MIT License](LICENSE). See the `LICENSE` file for more details.

## Acknowledgements

A huge thank you to Vue.js for its intuitive and powerful framework, ProseMirror for its robust text-editing capabilities, and Tailwind CSS for its flexible and efficient styling system. This project wouldn’t be the same without these amazing tools!