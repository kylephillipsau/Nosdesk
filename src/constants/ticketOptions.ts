// src/constants/ticketOptions.ts
export type TicketStatus = "open" | "in-progress" | "closed";
export type TicketPriority = "low" | "medium" | "high";

export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

export const STATUS_OPTIONS: SelectOption<TicketStatus>[] = [
  { value: "open", label: "Open" },
  { value: "in-progress", label: "In Progress" },
  { value: "closed", label: "Closed" },
];

export const PRIORITY_OPTIONS: SelectOption<TicketPriority>[] = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];