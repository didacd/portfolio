export interface GitHubUser {
	login: string;
	avatar_url: string;
	html_url: string;
	public_repos: number;
	followers: number;
	following: number;
	created_at: string;
}

export interface GitHubRepo {
	name: string;
	fork: boolean;
	stargazers_count: number;
	language: string | null;
}

export interface GitHubData {
	user: GitHubUser;
	stats: {
		repos: number;
		followers: number;
		following: number;
		stars: number;
	};
	languages: { name: string; count: number }[];
	memberSince: number;
}

const GITHUB_USERNAME = "didacd";
const API_BASE = "https://api.github.com";

async function gh<T>(path: string): Promise<T | null> {
	const headers: Record<string, string> = {
		Accept: "application/vnd.github+json",
		"User-Agent": "didac-portfolio",
	};

	const token = import.meta.env.GITHUB_TOKEN;
	if (token) {
		headers.Authorization = `Bearer ${token}`;
	}

	try {
		const res = await fetch(`${API_BASE}${path}`, { headers });
		if (!res.ok) {
			return null;
		}
		return (await res.json()) as T;
	} catch {
		return null;
	}
}

export async function getGitHubData(): Promise<GitHubData | null> {
	const [user, repos] = await Promise.all([
		gh<GitHubUser>(`/users/${GITHUB_USERNAME}`),
		gh<GitHubRepo[]>(`/users/${GITHUB_USERNAME}/repos?per_page=100`),
	]);

	if (!user) {
		return null;
	}

	const ownRepos = (repos ?? []).filter((repo) => !repo.fork);

	const stars = ownRepos.reduce(
		(total, repo) => total + repo.stargazers_count,
		0,
	);

	const languageCounts = new Map<string, number>();
	for (const repo of ownRepos) {
		if (repo.language) {
			languageCounts.set(
				repo.language,
				(languageCounts.get(repo.language) ?? 0) + 1,
			);
		}
	}

	const languages = [...languageCounts.entries()]
		.map(([name, count]) => ({ name, count }))
		.sort((a, b) => b.count - a.count)
		.slice(0, 5);

	return {
		user,
		stats: {
			repos: user.public_repos,
			followers: user.followers,
			following: user.following,
			stars,
		},
		languages,
		memberSince: new Date(user.created_at).getFullYear(),
	};
}
