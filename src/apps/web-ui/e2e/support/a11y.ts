// axe-core scan limited to WCAG 2.0 A/AA and 2.1 AA; returns serious and critical violations.
import AxeBuilder from "@axe-core/playwright";
import type { Page } from "@playwright/test";
import { saveJson } from "./evidence";

export const A11Y_TAGS = ["wcag2a", "wcag2aa", "wcag21aa"];

export type A11yViolation = {
  id: string;
  impact: string;
  help: string;
  nodes: string[];
};

export async function scanA11y(
  page: Page,
  evidence: { task: string; name: string },
  configure: (builder: AxeBuilder) => AxeBuilder = (builder) => builder,
): Promise<A11yViolation[]> {
  const results = await configure(
    new AxeBuilder({ page }).withTags(A11Y_TAGS),
  ).analyze();
  const violations = results.violations
    .filter(
      (violation) =>
        violation.impact === "serious" || violation.impact === "critical",
    )
    .map((violation) => ({
      id: violation.id,
      impact: violation.impact ?? "",
      help: violation.help,
      nodes: violation.nodes.map((node) => node.target.join(" ")),
    }));
  saveJson(evidence.task, `axe-${evidence.name}`, {
    url: page.url(),
    tags: A11Y_TAGS,
    violations,
  });
  return violations;
}
