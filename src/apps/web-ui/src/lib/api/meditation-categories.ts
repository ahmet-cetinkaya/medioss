import type { Category } from "./meditation-types";
import { request } from "./request";

export async function fetchCategories(): Promise<Category[]> {
  return request<Category[]>("/api/meditation/categories", undefined, {
    action: "categories.load",
  });
}

export async function renameCategory(id: number, name: string): Promise<void> {
  await request(
    `/api/meditation/categories/${id}`,
    { method: "PUT", body: JSON.stringify({ name }) },
    { action: "category.rename" },
  );
}

export async function deleteCategory(id: number): Promise<void> {
  await request(
    `/api/meditation/categories/${id}`,
    { method: "DELETE" },
    { action: "category.delete" },
  );
}

export async function createCategory(name: string): Promise<Category> {
  return request(
    "/api/meditation/categories",
    { method: "POST", body: JSON.stringify({ name }) },
    { action: "category.create" },
  );
}
