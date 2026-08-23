import { getIconData, iconToSVG } from "@iconify/utils";
import { icons as lucideIcons } from "@iconify-json/lucide";
import { fromHtml } from "hast-util-from-html";

export interface CalloutDefinition {
	icon: string;
	title: string;
	color: string;
}

export type CalloutDefinitions = Record<string, CalloutDefinition>;

type HastNode = {
	type: string;
	tagName?: string;
	properties?: Record<string, unknown>;
	value?: string;
	children?: HastNode[];
};

const calloutPattern = /^\s*\[!([a-z0-9-]+)\]\s*/i;

export function getCalloutDefinitions(config: unknown): CalloutDefinitions {
	if (
		!isRecord(config) ||
		!isRecord(config.profile) ||
		!isRecord(config.profile.callouts)
	) {
		throw new Error("src/config/site.yaml must define profile.callouts.");
	}

	const definitions: CalloutDefinitions = {};
	for (const [name, value] of Object.entries(config.profile.callouts)) {
		if (
			!isRecord(value) ||
			typeof value.icon !== "string" ||
			typeof value.title !== "string" ||
			typeof value.color !== "string"
		) {
			throw new Error(
				`Callout "${name}" must define an icon, title, and color.`,
			);
		}

		if (!getIconData(lucideIcons, value.icon)) {
			throw new Error(
				`Callout "${name}" references unknown Lucide icon "${value.icon}".`,
			);
		}

		definitions[name.toLowerCase()] = {
			icon: value.icon,
			title: value.title,
			color: value.color,
		};
	}

	return definitions;
}

export function rehypeCallouts(definitions: CalloutDefinitions) {
	return (tree: HastNode) => visit(tree, definitions);
}

function visit(node: HastNode, definitions: CalloutDefinitions) {
	if (node.type === "element" && node.tagName === "blockquote") {
		transformCallout(node, definitions);
	}

	for (const child of node.children ?? []) {
		visit(child, definitions);
	}
}

function transformCallout(node: HastNode, definitions: CalloutDefinitions) {
	const firstParagraph = node.children?.find(
		(child) => child.type === "element" && child.tagName === "p",
	);
	const firstText = firstParagraph?.children?.[0];
	if (
		!firstText ||
		firstText.type !== "text" ||
		typeof firstText.value !== "string"
	) {
		return;
	}

	const match = firstText.value.match(calloutPattern);
	if (!match) {
		return;
	}

	const type = match[1].toLowerCase();
	const definition = definitions[type];
	if (!definition) {
		return;
	}

	const [customTitle, ...bodyLines] = firstText.value
		.slice(match[0].length)
		.split(/\r?\n/);
	firstText.value = bodyLines.join("\n");
	const content = (node.children ?? []).filter(
		(child) => child !== firstParagraph || hasVisibleContent(firstParagraph),
	);

	node.tagName = "aside";
	node.properties = {
		className: ["callout"],
		"data-callout": type,
		style: `--callout-color: ${definition.color};`,
	};
	node.children = [
		{
			type: "element",
			tagName: "div",
			properties: { className: ["callout-header"] },
			children: [
				createIcon(definition.icon),
				{
					type: "element",
					tagName: "span",
					properties: { className: ["callout-title"] },
					children: [
						{ type: "text", value: customTitle.trim() || definition.title },
					],
				},
			],
		},
		{
			type: "element",
			tagName: "div",
			properties: { className: ["callout-content"] },
			children: content,
		},
	];
}

function createIcon(name: string): HastNode {
	const icon = getIconData(lucideIcons, name)!;
	const { attributes, body } = iconToSVG(icon, { height: "20", width: "20" });
	const tree = fromHtml(`<svg>${body}</svg>`, { fragment: true });
	const svg = tree.children[0] as HastNode;

	svg.properties = {
		...svg.properties,
		...attributes,
		"aria-hidden": "true",
		focusable: "false",
		className: ["callout-icon"],
	};
	return svg;
}

function hasVisibleContent(node: HastNode): boolean {
	return (
		node.children?.some(
			(child) => child.type !== "text" || child.value?.trim(),
		) ?? false
	);
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null;
}
