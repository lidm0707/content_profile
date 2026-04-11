# Supabase Initialization Tool

Binary tool for initializing Supabase database tables from CSV schema definitions.

## Requirements

- PostgreSQL database connection (DATABASE_URL environment variable)
- CSV schema files in `.supabase/tables/` directory
- SQL seed data file `.supabase/init_data.sql`
- CSV data files for upload in `.supabase/data/` directory

## Usage

### Basic Usage

Create tables from CSV schemas:

```bash
cargo run --bin supabase-init
```

### Drop Existing Tables

Drop all existing tables before recreation:

```bash
cargo run --bin supabase-init -- --drop
```

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
  - Example: `postgresql://postgres:password@localhost:5432/postgres`

## Seed Data

The `.supabase/init_data.sql` file contains sample data to populate the database after creating tables. It includes:

- Hierarchical tags structure (Technology, Business, Science, etc.)
- Sample content articles with different statuses (published, draft, archived)
- Content-tag associations linking articles to relevant tags

### Using Seed Data

After running the schema initialization, apply seed data:

```bash
# Method 1: Using psql
psql $DATABASE_URL -f .supabase/init_data.sql

# Method 2: Using Supabase CLI
supabase db reset --db-url $DATABASE_URL
# Then apply seed data
supabase db execute --file .supabase/init_data.sql --db-url $DATABASE_URL

# Method 3: Using cargo-run script (if configured)
cargo run --bin supabase-seed
```

### Seed Data Structure

**Tags:**
- Root categories: Technology, Lifestyle, Business, Science
- Subcategories with hierarchical parent-child relationships

**Content:**
- 10 sample articles covering Rust, Dioxus, web development, and programming topics
- Mixed statuses: published (7), draft (2), archived (1)

**Content-Tag Relations:**
- Each article associated with 2-3 relevant tags
- Proper foreign key references to maintain data integrity

### Reset and Reseed

To completely reset the database:

```bash
# Drop and recreate tables with seed data
cargo run --bin supabase-init -- --drop
psql $DATABASE_URL -f .supabase/init_data.sql
```

## CSV Data Upload

For batch data import using CSV files, use the files in `.supabase/data/` directory.

### Quick Start

```bash
# 1. Upload tags
psql $DATABASE_URL -c "\COPY tags(name,slug,parent_id,created_at,updated_at,synced_at) FROM '.supabase/data/tags_data.csv' WITH (FORMAT csv, HEADER)"

# 2. Upload content
psql $DATABASE_URL -c "\COPY content(title,slug,body,status,created_at,updated_at,synced_at) FROM '.supabase/data/content_data.csv' WITH (FORMAT csv, HEADER)"

# 3. Update tag hierarchy
psql $DATABASE_URL -f .supabase/data/update_tag_parents.sql

# 4. Import content-tags
psql $DATABASE_URL -f .supabase/data/import_content_tags.sql
```

### CSV Files

- `tags_data.csv` - 20 tags with hierarchical structure
- `content_data.csv` - 10 articles with different statuses
- `content_tags_data.csv` - Content-tag associations (uses slug references)
- `update_tag_parents.sql` -

## Schema File Format

CSV files in `.supabase/tables/` must follow this format:

```csv
table_name,column_name,data_type,nullable,default_value,constraints
content,id,INTEGER,NO,,PRIMARY KEY GENERATED ALWAYS AS IDENTITY
content,title,TEXT,NO,,
content,slug,TEXT,NO,,UNIQUE
```

## Features

- Parse CSV schema definitions
- Generate SQL CREATE TABLE statements
- Execute SQL against PostgreSQL database
- Optional DROP TABLE CASCADE for clean initialization
- Automatic table ordering based on dependencies

## Examples

### Initialize Database

```bash
export DATABASE_URL="postgresql://postgres:password@localhost:5432/postgres"
cargo run --bin supabase-init
```

### Clean Reinitialization

```bash
export DATABASE_URL="postgresql://postgres:password@localhost:5432/postgres"
cargo run --bin supabase-init -- --drop
```
