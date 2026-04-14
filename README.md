# Content Management System - Dioxus + Supabase

A modern, responsive content management system built with Rust's Dioxus framework and Supabase's powerful backend. This project demonstrates a full-stack application with real-time data management, clean architecture, and modern UI components.

## 🚀 Features

- **Content Management**: Create, read, update, and delete content items
- **Tag System**: Organize content with flexible tags and filter by tag
- **Rich User Interface**: Modern, responsive design built with Tailwind CSS
- **Real-time Data**: Seamless integration with Supabase database
- **Status Tracking**: Manage content with draft, published, and archived statuses
- **Automatic Slug Generation**: URL-friendly slugs generated from titles
- **Form Validation**: Client-side validation for all content inputs
- **Loading States**: Smooth loading indicators for async operations
- **Error Handling**: Comprehensive error handling and user feedback
- **Responsive Design**: Works seamlessly on desktop, tablet, and mobile devices
- **Modular Architecture**: Clean separation between SDK and UI layers

## 🛠 Technology Stack

- **Frontend**: Dioxus 0.7 (Rust-based reactive UI framework)
- **Backend**: Supabase (PostgreSQL + REST API + Real-time)
- **SDK**: Custom Rust SDK with business logic and models
- **Styling**: Tailwind CSS v4 (Utility-first CSS framework)
- **HTTP Client**: Reqwest (Async HTTP client for Rust)
- **Serialization**: Serde (Serialization/deserialization framework)
- **Date/Time**: Chrono (Date and time library)
- **Authentication**: Supabase Auth with JWT tokens

## 📁 Project Structure

```
content_profile/
├─ content_sdk/                # SDK library (reusable business logic)
│  ├─ src/
│  │  ├─ models/              # Data models and requests
│  │  │  ├─ mod.rs           # Models module
│  │  │  ├─ content.rs       # Content model and requests
│  │  │  ├─ tag.rs           # Tag model and requests
│  │  │  ├─ content_tag.rs   # Content-tag relationship model
│  │  │  └─ auth.rs          # Authentication models
│  │  ├─ services/            # Business logic services
│  │  │  ├─ mod.rs           # Services module
│  │  │  ├─ auth.rs          # Authentication service
│  │  │  ├─ content.rs       # Content service
│  │  │  ├─ tag.rs           # Tag service
│  │  │  ├─ supabase.rs      # Supabase client
│  │  │  ├─ local_storage.rs # Local storage service
│  │  │  └─ session.rs       # Session storage service
│  │  ├─ utils/               # Utilities
│  │  │  ├─ mod.rs           # Utils module
│  │  │  ├─ config.rs        # Configuration management
│  │  │  └─ markdown.rs      # Markdown processing utilities
│  │  ├─ hooks/              # Custom React-like hooks
│  │  │  ├─ mod.rs           # Hooks module
│  │  │  ├─ use_content.rs   # Content management hook
│  │  │  └─ use_tags.rs      # Tags management hook
│  │  └─ lib.rs              # SDK library entry point
│  └─ Cargo.toml              # SDK dependencies
├─ content_ui/                # UI application (Dioxus frontend)
│  ├─ assets/               # Static assets (images, CSS, etc.)
│  │  ├─ favicon.ico         # Application favicon
│  │  ├─ tailwind.css       # Tailwind CSS v4
│  │  └─ main.css           # Custom styles
│  ├─ src/
│  │  ├─ main.rs             # Application entry point
│  │  ├─ app.rs              # Root App component
│  │  ├─ routes.rs           # Route definitions
│  │  ├─ components/          # Reusable components
│  │  │  ├─ mod.rs           # Components module
│  │  │  ├─ navbar.rs        # Navigation bar
│  │  │  ├─ content_form.rs  # Content creation/editing form
│  │  │  ├─ content_list.rs  # List of content items
│  │  │  ├─ content_detail.rs # Detailed content view
│  │  │  ├─ stat_card.rs      # Statistics card component
│  │  │  └─ notification_card.rs # Notification component
│  │  ├─ pages/              # Page components
│  │  │  ├─ mod.rs           # Pages module
│  │  │  ├─ dashboard.rs      # Content management dashboard
│  │  │  ├─ content_edit.rs   # Content edit/create page
│  │  │  ├─ content_list.rs   # Content list page with tag filtering
│  │  │  ├─ login.rs          # Login page
│  │  │  ├─ tags_edit.rs     # Tag creation/editing page
│  │  │  └─ tags_list.rs     # Tags list page
│  │  ├─ contexts/            # Reactive state contexts
│  │  │  ├─ mod.rs           # Contexts module
│  │  │  ├─ user_context.rs  # User authentication context
│  │  │  ├─ content_context.rs # Content management context
│  │  │  └─ tag_context.rs  # Tag management context
│  │  └─ routes.rs           # Route definitions
│  ├─ Cargo.toml              # UI dependencies
│  ├─ build.rs                # Build script for environment variables
│  └─ Dioxus.toml            # Dioxus configuration
├─ supabase_client/            # Supabase client library
│  └─ Cargo.toml              # Supabase dependencies
├─ .env.example                # Environment variables template
├─ supabase_schema.sql         # Database schema for Supabase
└─ README.md                  # This file
```

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: Latest stable version (1.70+)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Node.js**: For Tailwind CSS (v16+)
  ```bash
  # Download from https://nodejs.org/
  ```
- **Dioxus CLI**: For serving the application
  ```bash
  curl -sSL http://dioxus.dev/install.sh | sh
  ```
- **Supabase Account**: Free account at https://supabase.com

## 🔧 Setup and Installation

### 1. Clone the Repository

```bash
git clone <your-repo-url>
cd content_profile
```

### 2. Install Dependencies

```bash
cargo check
```

This will automatically download and compile all required dependencies.

### 3. Configure Supabase

#### Create a Supabase Project

1. Go to [https://supabase.com](https://supabase.com)
2. Create a new project
3. Wait for the project to be ready (~2 minutes)
4. Navigate to Settings → API

#### Get Your Credentials

Copy the following values from your Supabase project:
- **Project URL**: From Settings → API → Project URL
- **anon/public key**: From Settings → API → anon/public key

#### Set Up Environment Variables

Copy the example environment file and add your credentials:

```bash
cp .env.example .env
```

Edit `.env` and add your Supabase credentials:

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key-here
```

⚠️ **Important**: Never commit `.env` to version control!

### 4. Create Database Schema

Run the SQL schema in your Supabase SQL Editor:

```bash
# Copy the contents of supabase_schema.sql
# Go to your Supabase dashboard
# Navigate to SQL Editor
# Paste and execute the schema
```

Or use the Supabase CLI:

```bash
supabase db reset --db-url "postgresql://postgres:[YOUR-PASSWORD]@db.[your-project].supabase.co:5432/postgres"
```

### 5. Set Up Row Level Security (RLS)

The schema includes RLS policies to protect your data. Ensure they're enabled:

```sql
-- In Supabase SQL Editor
ALTER TABLE content ENABLE ROW LEVEL SECURITY;
```

## 🚀 Running the Application

### Development Mode

Start the development server:

```bash
dx serve
```

This will:
- Compile your Rust code
- Start the Tailwind CSS compiler
- Launch a local web server (typically on http://localhost:8080)
- Enable hot reload for rapid development

### Build for Production

To build for web deployment:

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Desktop Application

To build as a desktop application:

```bash
dx serve --platform desktop
```

## 📊 Database Schema

The application uses a single `content` table with the following structure:

```sql
CREATE TABLE content (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    body TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Fields Description

- **id**: Auto-incrementing primary key
- **title**: Content title (required)
- **slug**: URL-friendly identifier (required, unique)
- **body**: Content body text (required)
- **status**: Content status ('draft', 'published', 'archived')
- **created_at**: Timestamp of content creation
- **updated_at**: Timestamp of last update

### Status Values

- `draft`: Content is in draft mode (not publicly visible)
- `published`: Content is published and publicly visible
- `archived`: Content is archived (hidden from public view)

## 🎯 Usage Guide

### 1. Home Page

The landing page provides an overview of the CMS and navigation to the dashboard.

### 2. Dashboard

The main content management interface displays:
- **Statistics**: Total content, published items, drafts, local only, and synced items
- **Tags**: Display all available tags as clickable badges
- **Content List**: Grid view of all content items with status indicators
- **Actions**: Create new content, refresh list, sync with server, edit existing content

#### Using Tags

- Click on any tag in the dashboard to filter content by that tag
- The content list page will display only content tagged with the selected tag
- Click "Dashboard" button to return to full content list
- Navigate to Tags List to create or manage tags

### 3. Creating Content

1. Navigate to Dashboard
2. Click "Create Content" button
3. Fill in the form:
   - **Title**: Content title (slug is auto-generated)
   - **Slug**: URL-friendly identifier (can be manually edited)
   - **Status**: Choose between Draft or Published
   - **Body**: Content text
   - **Tags**: Add tags to organize your content
4. Click "Create Content"

### 4. Editing Content

1. Navigate to Dashboard
2. Click "Edit" on any content card
3. Modify the content as needed
4. Click "Update Content"

### 5. Managing Tags

1. Navigate to Dashboard
2. Click on a tag to view content filtered by that tag
3. Navigate to Tags List from the navigation
4. Create new tags or edit existing tags
5. Add or remove tags from content items

### 6. Deleting Content

Currently, deletion is handled through the Supabase dashboard. Future versions will include in-app deletion.

## 🔌 API Reference

### Content Operations

All operations use the Supabase REST API with the following base URL:
```
https://your-project.supabase.co/rest/v1
```

#### Get All Content

```http
GET /content?order=created_at.desc
```

#### Get Content by ID

```http
GET /content?id=1
```

#### Get Content by Slug

```http
GET /content?slug=my-content-slug
```

#### Get Content by IDs

Batch fetch content items using the IN filter for efficient querying:

```http
GET /content?id=in.(1,2,3)
```

Use this when you need to fetch multiple content items by their IDs in a single request, which prevents N+1 query problems.

#### Get Content by Tags

Fetch all content items that have specific tags:

```http
# Step 1: Get content-tag junction records for a tag
GET /content_tags?tag_id=eq.5

# Step 2: Extract content_ids from response
# Example response: [{"id": 1, "content_id": 10, "tag_id": 5}, ...]
content_ids = [10, 20, 30]

# Step 3: Batch fetch content items
GET /content?id=in.(10,20,30)
```

This approach efficiently fetches all content for a specific tag using batch operations instead of N+1 queries.

#### Create Content

```http
POST /content
Content-Type: application/json

{
  "title": "My Content",
  "slug": "my-content",
  "body": "Content body text",
  "status": "draft"
}
```

#### Update Content

```http
PATCH /content?id=1
Content-Type: application/json

{
  "title": "Updated Title",
  "status": "published"
}
```

#### Delete Content

```http
DELETE /content?id=1
```

## 🧪 Testing

### Run Tests

```bash
cargo test
```

### Check Code Quality

```bash
cargo clippy
cargo fmt --check
```

## 🐛 Troubleshooting

### Common Issues

#### "Environment variable not set"

Ensure your `.env` file is properly configured with valid Supabase credentials.

#### "Failed to fetch content"

- Check your Supabase project status
- Verify the API key is correct
- Ensure Row Level Security allows public access (or adjust policies)
- Check the Supabase logs for errors

#### "Tailwind styles not loading"

- Ensure `tailwind.css` exists in the `assets` directory
- Restart the development server
- Check the browser console for CSS-related errors

#### "Build errors"

- Ensure you're using Rust 1.70 or later
- Run `cargo clean` then try building again
- Update dependencies with `cargo update`

### Debug Mode

Enable debug logging by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug dx serve
```

## 🔐 Security Considerations

1. **API Keys**: Never commit your Supabase API keys to version control
2. **Row Level Security**: Enable and configure RLS policies in Supabase
3. **Input Validation**: All user inputs are validated on the client side
4. **HTTPS**: Always use HTTPS in production
5. **Error Messages**: Don't expose sensitive information in error messages

## 🚦 Development Guidelines

### Code Style

- Use meaningful variable and function names
- Keep functions small and focused
- Document public APIs with comments
- Follow Rust naming conventions
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` to catch common issues

### Component Guidelines

- Components should be reusable and self-contained
- Use props for data passing
- Use signals for local state management
- Keep components focused on a single responsibility

### Adding New Features

1. Update the model in `src/models/`
2. Add service methods in `src/services/supabase.rs`
3. Create components in `src/components/`
4. Add routes in `src/routes.rs`
5. Create pages in `src/pages/`
6. Update the navigation in `src/components/navbar.rs`

## 🎨 Customization

### Styling

- Modify `assets/main.css` for custom styles
- Edit `assets/tailwind.css` for Tailwind configuration
- Customize colors, fonts, and spacing in components

### Database

- Modify `supabase_schema.sql` to add new tables or columns
- Update `src/models/` to reflect database changes
- Add corresponding service methods

### Routes

- Add new routes in `src/routes.rs`
- Create corresponding page components
- Update navigation as needed

## 📚 Additional Resources

- [Dioxus Documentation](https://dioxuslabs.com/learn/0.7/)
- [Supabase Documentation](https://supabase.com/docs)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Serde Documentation](https://serde.rs/)

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Ensure code passes `cargo clippy` and `cargo fmt`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## 📄 License

This project is open source and available under the MIT License.

## 🙏 Acknowledgments

- Dioxus team for the amazing framework
- Supabase for the excellent backend services
- Tailwind CSS for the utility-first CSS framework
- The Rust community for valuable tools and libraries

## 📞 Support

For issues, questions, or contributions:

- Open an issue on GitHub
- Join the Dioxus Discord community
- Check the Supabase documentation

## 🗺 Roadmap

### Recently Completed ✅

- [x] User authentication and authorization
- [x] Content categories/tags
- [x] Tag-based content filtering
- [x] Dashboard with statistics
- [x] Sync functionality (local/offline mode)
- [x] Content CRUD operations
- [x] Tag CRUD operations
- [x] Reactive state management with contexts

### Planned Features

- [ ] Image upload functionality
- [ ] Rich text editor integration
- [ ] Search functionality
- [ ] Content versioning
- [ ] Dark mode support
- [ ] Multi-language support
- [ ] Content scheduling
- [ ] SEO optimization features
- [ ] Offline mode support
- [ ] Data synchronization with remote server

### Performance Improvements

- [ ] Implement caching strategy
- [ ] Optimize database queries
- [ ] Add pagination for content lists
- [ ] Implement lazy loading for images
- [ ] Optimize WASM bundle size

### Developer Experience

- [ ] Add comprehensive unit tests
- [ ] Add integration tests
- [ ] Set up CI/CD pipeline
- [ ] Add API documentation
- [ ] Create example templates
- [ ] Add storybook for components
- [ ] Improve error messages

---

**Built with ❤️ using Rust, Dioxus, and Supabase**