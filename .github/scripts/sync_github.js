const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Automatically load local .env variables if running locally (Dependency-Free!)
const envPath = path.join(process.cwd(), '.env');
if (fs.existsSync(envPath)) {
    fs.readFileSync(envPath, 'utf-8').split(/\r?\n/).forEach(line => {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#')) {
            const idx = trimmed.indexOf('=');
            if (idx !== -1) {
                const k = trimmed.substring(0, idx).trim();
                let v = trimmed.substring(idx + 1).trim();
                if (v.startsWith('"') && v.endsWith('"')) v = v.slice(1, -1);
                if (v.startsWith("'") && v.endsWith("'")) v = v.slice(1, -1);
                if (!process.env[k]) {
                    process.env[k] = v;
                }
            }
        }
    });
}

// 1. Get GitHub credentials & repository info
let token = process.env.GITHUB_TOKEN;
if (!token) {
    console.error("❌ GITHUB_TOKEN environment variable is missing. Synchronization aborted.");
    process.exit(1);
}

// Clean up any trailing/leading whitespaces or quote artifacts from the token
token = token.trim();

// Log token diagnostic info (secure, only prefix and length)
const tokenPrefix = token.substring(0, 4);
console.log(`🔑 Diagnostics: Token prefix="${tokenPrefix}", length=${token.length}`);

let repository = process.env.GITHUB_REPOSITORY; // Injected automatically by GitHub Actions
if (!repository) {
    try {
        const remoteUrl = execSync('git config --get remote.origin.url').toString().trim();
        // Match formats: git@github.com:owner/repo.git or https://github.com/owner/repo.git
        const match = remoteUrl.match(/github\.com[:/]([^/]+\/[^.]+)(?:\.git)?$/);
        if (match && match[1]) {
            repository = match[1];
        }
    } catch (e) {
        console.warn("⚠️ Could not extract repository path from local git config. Fallback to manual setup.");
    }
}

if (!repository) {
    console.error("❌ Could not identify the GitHub repository path (owner/repo). Please set GITHUB_REPOSITORY.");
    process.exit(1);
}

const [owner, repoName] = repository.split('/');
const isPullMode = process.argv.includes('--pull') || process.argv.includes('-p');
console.log(`🛰️ Mode: ${isPullMode ? 'PULL (Remote ──> Local)' : 'PUSH (Local ──> Remote)'}`);
console.log(`📂 Repository: https://github.com/${owner}/${repoName}`);

const BASE_URL = `https://api.github.com/repos/${owner}/${repoName}`;

// COMPATIBILITY WORKAROUND: Classic Personal Access Tokens (PATs) starting with ghp_ or older Classic Tokens
// require 'token <token>' authorization header on classical GitHub APIs to prevent 404 unauthorized errors.
const authHeader = (token.startsWith('ghp_') || token.startsWith('github_pat_'))
    ? `token ${token}`
    : `Bearer ${token}`;

const HEADERS = {
    'Authorization': authHeader,
    'Accept': 'application/vnd.github+json',
    'X-GitHub-Api-Version': '2022-11-28',
    'User-Agent': 'Digi-Life-Sync-Script'
};

// Simple, robust YAML Frontmatter parser & serializer (Dependency-Free!)
function parseTicket(filePath) {
    const rawContent = fs.readFileSync(filePath, 'utf-8');
    const frontmatterRegex = /^---\r?\n([\s\S]+?)\r?\n---\r?\n([\s\S]*)$/;
    const match = rawContent.match(frontmatterRegex);
    if (!match) return null;

    const yamlStr = match[1];
    const bodyStr = match[2].trim();
    const metadata = {};

    yamlStr.split('\n').forEach(line => {
        const idx = line.indexOf(':');
        if (idx !== -1) {
            const key = line.substring(0, idx).trim();
            let val = line.substring(idx + 1).trim();
            // Clean up quotes
            if (val.startsWith('"') && val.endsWith('"')) val = val.slice(1, -1);
            if (val.startsWith("'") && val.endsWith("'")) val = val.slice(1, -1);
            // Handle arrays
            if (val.startsWith('[') && val.endsWith(']')) {
                try {
                    val = JSON.parse(val.replace(/'/g, '"'));
                } catch(e) {}
            }
            metadata[key] = val;
        }
    });

    return { metadata, body: bodyStr, raw_path: filePath, filename: path.basename(filePath) };
}

function stringifyTicket(metadata, body) {
    let yamlLines = ['---'];
    Object.keys(metadata).forEach(key => {
        if (key === 'folder_status' || key === 'raw_path' || key === 'filename') return; // Skip temporary fields
        let val = metadata[key];
        if (Array.isArray(val)) {
            yamlLines.push(`${key}: ${JSON.stringify(val)}`);
        } else if (typeof val === 'string' && (val.includes(':') || val.includes('#') || val.includes('"') || val.includes("'"))) {
            yamlLines.push(`${key}: "${val.replace(/"/g, '\\"')}"`);
        } else {
            yamlLines.push(`${key}: ${val}`);
        }
    });
    yamlLines.push('---');
    return yamlLines.join('\n') + '\n\n' + body + '\n';
}

// 2. Scan all local tickets across status directories
function getLocalTickets() {
    const ticketDir = path.join(process.cwd(), '.tickets');
    const statuses = ['open', 'ongoing', 'closed'];
    const tickets = [];

    statuses.forEach(status => {
        const dirPath = path.join(ticketDir, status);
        if (!fs.existsSync(dirPath)) return;

        const files = fs.readdirSync(dirPath).filter(file => file.endsWith('.md'));
        files.forEach(file => {
            const parsed = parseTicket(path.join(dirPath, file));
            if (parsed) {
                parsed.metadata.folder_status = status;
                tickets.push(parsed);
            }
        });
    });

    return tickets;
}

// 3. Make REST API helper calls via native fetch
async function apiRequest(endpoint, options = {}) {
    const url = endpoint.startsWith('http') ? endpoint : `${BASE_URL}${endpoint}`;
    const response = await fetch(url, {
        headers: HEADERS,
        ...options
    });

    if (!response.ok) {
        const errText = await response.text();
        throw new Error(`API Error on ${url} (${response.status}): ${errText}`);
    }

    if (response.status === 204) return null;
    return await response.json();
}

// GraphQL Helper (Strictly requires Bearer header)
async function graphqlRequest(query, variables = {}) {
    const response = await fetch('https://api.github.com/graphql', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Accept': 'application/json',
            'User-Agent': 'Digi-Life-Sync-Script'
        },
        body: JSON.stringify({ query, variables })
    });

    const data = await response.json();
    if (!response.ok || data.errors) {
        const errs = data.errors ? data.errors.map(e => e.message).join(', ') : 'HTTP error';
        throw new Error(`GraphQL Error: ${errs}`);
    }
    return data.data;
}

// Helper to sanitize filenames
function sanitizeSlug(title) {
    return title.toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/(^-|-$)/g, '');
}

// 4. Main PUSH Loop (Local ──> Remote)
async function runPush(localTickets, githubIssues) {
    console.log(`📦 Starting PUSH Synchronization to GitHub...`);
    for (const local of localTickets) {
        const meta = local.metadata;
        const expectedTitlePrefix = `[${meta.id}]`;
        const fullTitle = `${expectedTitlePrefix} ${meta.title}`;

        // Map folder status to GitHub states and labels
        const githubState = meta.folder_status === 'closed' ? 'closed' : 'open';
        const expectedLabels = [
            `priority:${meta.priority || 'medium'}`,
            `status:${meta.folder_status}`
        ];
        if (meta.epic_phase) {
            const epicSlug = sanitizeSlug(meta.epic_phase);
            expectedLabels.push(`epic:${epicSlug}`);
        }

        // Look for matching issue on GitHub
        const existingIssue = githubIssues.find(issue => issue.title.startsWith(expectedTitlePrefix));

        if (!existingIssue) {
            console.log(`➕ Creating new Issue on GitHub: "${fullTitle}"...`);
            const payload = {
                title: fullTitle,
                body: local.body,
                labels: expectedLabels
            };
            const newIssue = await apiRequest('/issues', {
                method: 'POST',
                body: JSON.stringify(payload)
            });
            console.log(`   └─ Successfully created: Issue #${newIssue.number}`);

            // ATOMIC WORKAROUND: GitHub REST API creates issues as "open" by default on creation.
            // If the ticket is locally closed, we must immediately follow up with a PATCH to close it!
            if (githubState === 'closed') {
                console.log(`   └─ Local ticket is closed. Instantly closing Issue #${newIssue.number} on GitHub...`);
                await apiRequest(`/issues/${newIssue.number}`, {
                    method: 'PATCH',
                    body: JSON.stringify({ state: 'closed' })
                });
            }
        } else {
            let needsUpdate = false;
            const updatePayload = {};

            if (existingIssue.title !== fullTitle) {
                updatePayload.title = fullTitle;
                needsUpdate = true;
            }

            if (existingIssue.state !== githubState) {
                updatePayload.state = githubState;
                needsUpdate = true;
            }

            if (existingIssue.body !== local.body) {
                updatePayload.body = local.body;
                needsUpdate = true;
            }

            // High-precision semantic label diff check
            const currentLabels = existingIssue.labels.map(l => l.name);
            const managedCurrentLabels = currentLabels.filter(l => l.startsWith('priority:') || l.startsWith('status:') || l.startsWith('epic:'));
            
            const labelsChanged = expectedLabels.some(l => !currentLabels.includes(l)) || 
                                  managedCurrentLabels.some(l => !expectedLabels.includes(l));
            if (labelsChanged) {
                updatePayload.labels = expectedLabels;
                needsUpdate = true;
            }

            if (needsUpdate) {
                console.log(`🔄 Updating Issue #${existingIssue.number}: "${fullTitle}"...`);
                await apiRequest(`/issues/${existingIssue.number}`, {
                    method: 'PATCH',
                    body: JSON.stringify(updatePayload)
                });
                console.log(`   └─ Successfully updated.`);
            } else {
                console.log(`🟢 Issue #${existingIssue.number} ("${meta.title}") is already in sync.`);
            }
        }
    }
}

// 5. Main PULL Loop (Remote ──> Local)
async function runPull(localTickets, githubIssues) {
    console.log(`📥 Starting PULL Synchronization to Local Workspace...`);
    const ticketDir = path.join(process.cwd(), '.tickets');

    // Group local tickets by ID for quick lookup
    const localMap = {};
    localTickets.forEach(t => {
        localMap[t.metadata.id] = t;
    });

    for (const issue of githubIssues) {
        // Parse Title: "[TCK-101] Title" (Now completely status-neutral!)
        const titleMatch = issue.title.match(/^\[(TCK-\d+)\]\s*(.*)$/);
        if (!titleMatch) {
            // Ignore non-ticket issues
            continue;
        }

        const ticketId = titleMatch[1];
        const ticketTitle = titleMatch[2].trim();

        // Extract metadata from labels
        let priority = 'medium';
        let folderStatus = issue.state === 'closed' ? 'closed' : 'open';
        let epicPhase = '';

        issue.labels.forEach(labelObj => {
            const label = labelObj.name;
            if (label.startsWith('priority:')) priority = label.substring(9);
            if (label.startsWith('status:')) folderStatus = label.substring(7);
            if (label.startsWith('epic:')) epicPhase = label.substring(5);
        });

        // Double check state alignment
        if (issue.state === 'closed' && folderStatus !== 'closed') {
            folderStatus = 'closed';
        } else if (issue.state === 'open' && folderStatus === 'closed') {
            folderStatus = 'open';
        }

        const localTicket = localMap[ticketId];

        if (localTicket) {
            // UPDATE LOCAL TICKET
            const meta = localTicket.metadata;
            let needsMove = meta.folder_status !== folderStatus;
            let needsUpdate = needsMove || 
                              meta.title !== ticketTitle || 
                              meta.priority !== priority || 
                              localTicket.body !== issue.body;

            if (needsUpdate) {
                console.log(`🔄 Updating Local Ticket ${ticketId} based on GitHub Issue #${issue.number}...`);
                
                // Keep existing local metadata that isn't on GitHub
                const updatedMetadata = {
                    ...meta,
                    title: ticketTitle,
                    priority: priority,
                    status: folderStatus === 'closed' ? 'closed' : (folderStatus === 'ongoing' ? 'ongoing' : 'open')
                };

                if (folderStatus === 'closed' && !updatedMetadata.completed) {
                    updatedMetadata.completed = new Date().toISOString().split('T')[0];
                }

                const serializedContent = stringifyTicket(updatedMetadata, issue.body);
                let targetPath = localTicket.raw_path;

                if (needsMove) {
                    // Use git mv or standard fs.rename
                    const newDir = path.join(ticketDir, folderStatus);
                    if (!fs.existsSync(newDir)) fs.mkdirSync(newDir, { recursive: true });

                    // Clean filename slug if title changed
                    const cleanSlug = sanitizeSlug(ticketTitle);
                    const newFilename = `${ticketId}-${cleanSlug}.md`;
                    const newPath = path.join(newDir, newFilename);

                    console.log(`   └─ Moving file: ${localTicket.filename} ──> ${folderStatus}/${newFilename}`);
                    try {
                        execSync(`git mv "${localTicket.raw_path}" "${newPath}" 2>/dev/null || mv "${localTicket.raw_path}" "${newPath}"`);
                        targetPath = newPath;
                    } catch (e) {
                        fs.renameSync(localTicket.raw_path, newPath);
                        targetPath = newPath;
                    }
                }

                fs.writeFileSync(targetPath, serializedContent, 'utf-8');
                console.log(`   └─ Local file successfully updated and saved.`);
            } else {
                console.log(`🟢 Local Ticket ${ticketId} is already in sync with GitHub.`);
            }
        } else {
            // CREATE NEW LOCAL TICKET (Someone created an Issue directly on GitHub!)
            console.log(`➕ Creating new local Ticket for GitHub Issue #${issue.number} ("${ticketTitle}")...`);
            
            const newMetadata = {
                id: ticketId,
                title: ticketTitle,
                epic_phase: epicPhase || "Unassigned",
                status: folderStatus === 'closed' ? 'closed' : (folderStatus === 'ongoing' ? 'ongoing' : 'open'),
                priority: priority,
                created: new Date().toISOString().split('T')[0]
            };

            if (folderStatus === 'closed') {
                newMetadata.completed = new Date().toISOString().split('T')[0];
            }

            const serializedContent = stringifyTicket(newMetadata, issue.body || "No description provided.");
            const targetDir = path.join(ticketDir, folderStatus);
            if (!fs.existsSync(targetDir)) fs.mkdirSync(targetDir, { recursive: true });

            const cleanSlug = sanitizeSlug(ticketTitle);
            const filename = `${ticketId}-${cleanSlug}.md`;
            const targetPath = path.join(targetDir, filename);

            fs.writeFileSync(targetPath, serializedContent, 'utf-8');
            try {
                execSync(`git add "${targetPath}"`);
            } catch (e) {}
            console.log(`   └─ Created local file: .tickets/${folderStatus}/${filename}`);
        }
    }
}

// 6. Bootstrap Exec
async function run() {
    try {
        console.log("📥 Fetching active and closed issues from GitHub...");
        let githubIssues = [];
        let page = 1;
        while (true) {
            const issues = await apiRequest(`/issues?state=all&per_page=100&page=${page}`);
            if (!issues || issues.length === 0) break;
            githubIssues = githubIssues.concat(issues);
            if (issues.length < 100) break;
            page++;
        }
        console.log(`✅ Loaded ${githubIssues.length} issues from GitHub.`);

        const localTickets = getLocalTickets();
        console.log(`📦 Loaded ${localTickets.length} local tickets from .tickets/.`);

        if (isPullMode) {
            await runPull(localTickets, githubIssues);
            
            // Re-import local tickets after pulling and update the Index Register
            console.log("📝 Re-building the central Index Register (EPIC_CONSOLIDATION_BACKLOG.md)...");
            const freshLocalTickets = getLocalTickets();
            const backlogPath = path.join(process.cwd(), 'docs', 'EPIC_CONSOLIDATION_BACKLOG.md');
            if (fs.existsSync(backlogPath)) {
                let backlogContent = fs.readFileSync(backlogPath, 'utf-8');
                
                // Parse and rebuild table rows
                let tableHeaderIdx = backlogContent.indexOf('| Ticket-ID | Titel |');
                if (tableHeaderIdx !== -1) {
                    const beforeTable = backlogContent.substring(0, tableHeaderIdx);
                    let afterTableIdx = backlogContent.indexOf('---', tableHeaderIdx + 100);
                    if (afterTableIdx === -1) afterTableIdx = backlogContent.length;
                    const afterTable = backlogContent.substring(afterTableIdx);

                    // Rebuild Table
                    const tableRows = [
                        '| Ticket-ID | Titel | Epic / Phase | Status | Prio | Ticket-Datei | Verknüpfte Quell-Ressource |',
                        '| :--- | :--- | :--- | :--- | :--- | :--- | :--- |'
                    ];

                    freshLocalTickets.sort((a,b) => {
                        const idA = parseInt(a.metadata.id.match(/\d+/)[0]);
                        const idB = parseInt(b.metadata.id.match(/\d+/)[0]);
                        // Closed first, then Todo
                        const isAClosed = a.metadata.folder_status === 'closed';
                        const isBClosed = b.metadata.folder_status === 'closed';
                        if (isAClosed && !isBClosed) return -1;
                        if (!isAClosed && isBClosed) return 1;
                        return idA - idB;
                    }).forEach(t => {
                        const m = t.metadata;
                        const statusBadge = `\`${m.status}\``;
                        const linkStr = `[Link](../.tickets/${m.folder_status}/${m.filename})`;
                        let resourceStr = '-';
                        if (m.dependencies && m.dependencies.length > 0) {
                            resourceStr = m.dependencies.map(d => `${d}`).join(', ');
                        }
                        tableRows.push(`| **${m.id}** | ${m.title} | ${m.epic_phase} | ${statusBadge} | \`${m.priority || 'medium'}\` | ${linkStr} | ${resourceStr} |`);
                    });

                    const updatedBacklog = beforeTable + tableRows.join('\n') + '\n\n' + afterTable;
                    fs.writeFileSync(backlogPath, updatedBacklog, 'utf-8');
                    console.log(`   └─ Successfully updated index register.`);
                }
            }
        } else {
            await runPush(localTickets, githubIssues);
        }

        console.log("🎉 Sync finished successfully!");
    } catch (error) {
        console.error("❌ Sync failed with exception:", error.message);
        process.exit(1);
    }
}

run();
