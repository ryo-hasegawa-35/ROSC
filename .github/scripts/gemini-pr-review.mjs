import process from "node:process";

const [owner, repo] = (process.env.GITHUB_REPOSITORY || "").split("/");
const githubToken = process.env.GITHUB_TOKEN;
const geminiApiKey = process.env.GEMINI_API_KEY;
const triggerUser = process.env.REVIEW_TRIGGER_USER || owner;
const reviewModel = process.env.REVIEW_MODEL || "gemini-2.5-flash";
const maxFiles = Number.parseInt(process.env.REVIEW_MAX_FILES || "25", 10);
const maxPatchChars = Number.parseInt(
  process.env.REVIEW_MAX_PATCH_CHARS || "60000",
  10,
);
const manualPrNumber = Number.parseInt(process.env.INPUT_PR_NUMBER || "", 10);
const force = String(process.env.INPUT_FORCE || "false").toLowerCase() === "true";

if (!owner || !repo || !githubToken) {
  throw new Error("Missing GitHub repository context or token.");
}

if (!geminiApiKey) {
  console.log("GEMINI_API_KEY is not configured. Skipping review assistant.");
  process.exit(0);
}

const githubApiBase = "https://api.github.com";
const githubHeaders = {
  Accept: "application/vnd.github+json",
  Authorization: `Bearer ${githubToken}`,
  "X-GitHub-Api-Version": "2022-11-28",
};

async function github(path, init = {}) {
  const response = await fetch(`${githubApiBase}${path}`, {
    ...init,
    headers: {
      ...githubHeaders,
      ...(init.headers || {}),
    },
  });

  if (!response.ok) {
    const body = await response.text();
    throw new Error(`GitHub API error ${response.status} on ${path}: ${body}`);
  }

  if (response.status === 204) {
    return null;
  }

  return response.json();
}

async function githubPaged(path) {
  const items = [];
  for (let page = 1; page <= 10; page += 1) {
    const separator = path.includes("?") ? "&" : "?";
    const batch = await github(`${path}${separator}per_page=100&page=${page}`);
    items.push(...batch);
    if (batch.length < 100) {
      break;
    }
  }
  return items;
}

function truncate(text, maxLength) {
  if (!text) {
    return "";
  }
  return text.length <= maxLength ? text : `${text.slice(0, maxLength)}\n... [truncated]`;
}

function buildDiffDigest(files) {
  const selected = files.slice(0, maxFiles);
  let remainingPatchBudget = maxPatchChars;

  return selected.map((file) => {
    let patch = file.patch || "";
    if (patch.length > remainingPatchBudget) {
      patch = truncate(patch, Math.max(0, remainingPatchBudget));
    }
    remainingPatchBudget = Math.max(0, remainingPatchBudget - patch.length);

    return [
      `FILE: ${file.filename}`,
      `STATUS: ${file.status}`,
      `ADDITIONS: ${file.additions}`,
      `DELETIONS: ${file.deletions}`,
      `CHANGES: ${file.changes}`,
      patch ? `PATCH:\n${patch}` : "PATCH: [not available or omitted]",
    ].join("\n");
  }).join("\n\n---\n\n");
}

function parseJsonResponse(text) {
  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`Failed to parse Gemini JSON response: ${error.message}\n${text}`);
  }
}

async function callGemini(prompt) {
  const response = await fetch(
    `https://generativelanguage.googleapis.com/v1beta/models/${reviewModel}:generateContent`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-goog-api-key": geminiApiKey,
      },
      body: JSON.stringify({
        contents: [
          {
            role: "user",
            parts: [{ text: prompt }],
          },
        ],
        generationConfig: {
          responseMimeType: "application/json",
          responseJsonSchema: {
            type: "object",
            properties: {
              overview: { type: "string" },
              change_summary: {
                type: "array",
                items: { type: "string" },
              },
              good_points: {
                type: "array",
                items: { type: "string" },
              },
              concerns: {
                type: "array",
                items: {
                  type: "object",
                  properties: {
                    title: { type: "string" },
                    severity: {
                      type: "string",
                      enum: ["low", "medium", "high"],
                    },
                    detail: { type: "string" },
                    files: {
                      type: "array",
                      items: { type: "string" },
                    },
                  },
                  required: ["title", "severity", "detail", "files"],
                },
              },
              risks: {
                type: "array",
                items: { type: "string" },
              },
              suggestions: {
                type: "array",
                items: { type: "string" },
              },
            },
            required: [
              "overview",
              "change_summary",
              "good_points",
              "concerns",
              "risks",
              "suggestions",
            ],
          },
        },
      }),
    },
  );

  if (!response.ok) {
    const body = await response.text();
    throw new Error(`Gemini API error ${response.status}: ${body}`);
  }

  const payload = await response.json();
  const text = payload.candidates?.[0]?.content?.parts?.[0]?.text;
  if (!text) {
    throw new Error(`Gemini response did not include text: ${JSON.stringify(payload)}`);
  }

  return parseJsonResponse(text);
}

function formatList(items, emptyText = "- 特になし") {
  if (!items || items.length === 0) {
    return emptyText;
  }
  return items.map((item) => `- ${item}`).join("\n");
}

function formatConcerns(concerns) {
  if (!concerns || concerns.length === 0) {
    return "- 大きな懸念は見つかりませんでした。";
  }
  return concerns
    .map((concern) => {
      const files = concern.files?.length ? ` 対象: ${concern.files.join(", ")}` : "";
      return `- [${concern.severity}] ${concern.title}: ${concern.detail}${files}`;
    })
    .join("\n");
}

function buildCommentBody({ reactionId, sha, review }) {
  return `<!-- gemini-pr-review reaction:${reactionId} sha:${sha} -->
### Gemini レビュー補助

困惑リアクションを検知したので、日本語で補助レビューをまとめました。  
この bot は**コメント支援専用**で、承認やマージは行いません。

**ざっくり概要**

${review.overview}

**変更内容**

${formatList(review.change_summary)}

**良い点**

${formatList(review.good_points)}

**気になる点**

${formatConcerns(review.concerns)}

**リスク**

${formatList(review.risks)}

**補足提案**

${formatList(review.suggestions)}
`;
}

function buildPrompt({ pr, files }) {
  const diffDigest = buildDiffDigest(files);
  const fileNotice =
    files.length > maxFiles
      ? `注意: 変更ファイルは ${files.length} 件ありますが、レビュー入力には先頭 ${maxFiles} 件を優先的に含めています。`
      : `変更ファイル数: ${files.length} 件です。`;

  return `あなたは GitHub の pull request を補助レビューする日本語レビュー bot です。

次の pull request を読んで、日本語で簡潔かつ実務的なレビュー補助を作成してください。

要件:
- 出力は JSON のみ
- 「良い点」と「気になる点」の両方を書く
- 気になる点は、重大度を low / medium / high のいずれかで付ける
- 変更概要は 3〜6 個程度の箇条書きにする
- 推測だけで断定しない
- マージ判断はしない
- 口調は落ち着いた日本語
- docs-only の変更なら、無理にバグを捏造しない
- 気になる点がない場合は concerns を空配列にしてよい

PR 情報:
- 番号: #${pr.number}
- タイトル: ${pr.title}
- 作成者: ${pr.user.login}
- ベース: ${pr.base.ref}
- ヘッド: ${pr.head.ref}
- HEAD SHA: ${pr.head.sha}
- URL: ${pr.html_url}

PR 本文:
${pr.body || "(empty)"}

${fileNotice}

変更ファイルと差分:
${diffDigest || "(no file diff available)"}
`;
}

async function findProcessedComment(prNumber, reactionId) {
  const comments = await githubPaged(`/repos/${owner}/${repo}/issues/${prNumber}/comments`);
  return comments.find((comment) =>
    comment.body?.includes(`<!-- gemini-pr-review reaction:${reactionId} `),
  );
}

async function addThumbsUpReaction(prNumber) {
  await github(`/repos/${owner}/${repo}/issues/${prNumber}/reactions`, {
    method: "POST",
    body: JSON.stringify({ content: "+1" }),
  });
}

async function postComment(prNumber, body) {
  await github(`/repos/${owner}/${repo}/issues/${prNumber}/comments`, {
    method: "POST",
    body: JSON.stringify({ body }),
  });
}

async function loadPullRequests() {
  if (Number.isInteger(manualPrNumber)) {
    const pr = await github(`/repos/${owner}/${repo}/pulls/${manualPrNumber}`);
    return [pr];
  }

  return githubPaged(`/repos/${owner}/${repo}/pulls?state=open&sort=updated&direction=desc`);
}

async function getTriggerReaction(prNumber) {
  const reactions = await githubPaged(`/repos/${owner}/${repo}/issues/${prNumber}/reactions`);

  const ownReaction = reactions
    .filter((reaction) => reaction.content === "confused")
    .filter((reaction) => reaction.user?.login === triggerUser)
    .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))[0];

  return ownReaction || null;
}

async function processPullRequest(pr) {
  const trigger =
    (await getTriggerReaction(pr.number)) ||
    (Number.isInteger(manualPrNumber) && force
      ? {
          id: `manual-${pr.head.sha}`,
          synthetic: true,
        }
      : null);

  if (!trigger) {
    return false;
  }

  const processedComment = await findProcessedComment(pr.number, trigger.id);
  if (processedComment && !force) {
    console.log(
      `Skipping PR #${pr.number}: trigger ${trigger.id} already processed in comment ${processedComment.id}.`,
    );
    return false;
  }

  if (!trigger.synthetic) {
    await addThumbsUpReaction(pr.number);
  }

  const files = await githubPaged(`/repos/${owner}/${repo}/pulls/${pr.number}/files`);
  const review = await callGemini(buildPrompt({ pr, files }));
  const commentBody = buildCommentBody({
    reactionId: trigger.id,
    sha: pr.head.sha,
    review,
  });
  await postComment(pr.number, commentBody);

  console.log(`Posted Gemini review helper comment on PR #${pr.number}.`);
  return true;
}

async function main() {
  const pullRequests = await loadPullRequests();
  let processed = 0;

  for (const pr of pullRequests) {
    try {
      if (await processPullRequest(pr)) {
        processed += 1;
      }
    } catch (error) {
      console.error(`Failed to process PR #${pr.number}:`, error);
    }
  }

  console.log(`Gemini review assistant finished. Processed ${processed} pull request(s).`);
}

await main();
