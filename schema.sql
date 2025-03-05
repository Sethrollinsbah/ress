CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT,
    email TEXT NOT NULL UNIQUE,
    auth_method TEXT NOT NULL,
    profile_picture TEXT,
    bio TEXT,
    location TEXT,
    website TEXT,
    created_at INTEGER NOT NULL,
    last_login INTEGER,
    is_active BOOLEAN NOT NULL,
    verification_status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT,
    start_time INTEGER, 
    expiration_time INTEGER,
    ip_address TEXT,
    user_agent TEXT, 
    is_authenticated BOOLEAN NOT NULL,
    roles TEXT,
    language TEXT, 
    last_active INTEGER
);

CREATE TABLE IF NOT EXISTS subscribers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    opted_in BOOLEAN DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS mailing_lists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS subscriptions (
    subscriber_id INTEGER NOT NULL,
    mailing_list_id INTEGER NOT NULL,
    subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (subscriber_id, mailing_list_id),
    FOREIGN KEY (subscriber_id) REFERENCES subscribers(id),
    FOREIGN KEY (mailing_list_id) REFERENCES mailing_lists(id)
);

CREATE TABLE IF NOT EXISTS appointments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    appointment_date TEXT NOT NULL, 
    appointment_time TEXT NOT NULL, 
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'scheduled',
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    url TEXT UNIQUE NOT NULL,
    content TEXT,
    FOREIGN KEY (domain_id) REFERENCES domains(id)
);

CREATE TABLE IF NOT EXISTS keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS keyword_occurrences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL,
    keyword_id INTEGER NOT NULL,
    frequency INTEGER NOT NULL,
    FOREIGN KEY (page_id) REFERENCES pages(id),
    FOREIGN KEY (keyword_id) REFERENCES keywords(id)
);
