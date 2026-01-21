/**
 * Formatting utilities for dates and text.
 */

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/**
 * Format a date as relative time (e.g., "2 hours ago", "Yesterday").
 * Switches to absolute format after 7 days.
 *
 * @param dateString - ISO 8601 date string
 * @returns Formatted relative or absolute date string
 */
export function formatRelativeDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();

  // Handle invalid dates
  if (isNaN(date.getTime())) {
    return "Invalid date";
  }

  const diff = now.getTime() - date.getTime();

  // Handle future dates
  if (diff < 0) {
    return formatAbsoluteDate(dateString);
  }

  // Less than a minute
  if (diff < MINUTE) {
    return "Just now";
  }

  // Less than an hour
  if (diff < HOUR) {
    const minutes = Math.floor(diff / MINUTE);
    return minutes === 1 ? "1 minute ago" : `${minutes} minutes ago`;
  }

  // Less than a day
  if (diff < DAY) {
    const hours = Math.floor(diff / HOUR);
    return hours === 1 ? "1 hour ago" : `${hours} hours ago`;
  }

  // Check if yesterday
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (isSameDay(date, yesterday)) {
    return "Yesterday";
  }

  // Less than 7 days
  if (diff < 7 * DAY) {
    const days = Math.floor(diff / DAY);
    return days === 1 ? "1 day ago" : `${days} days ago`;
  }

  // More than 7 days - use absolute format
  return formatAbsoluteDate(dateString);
}

/**
 * Format a date as absolute date (e.g., "Jan 15, 2025").
 *
 * @param dateString - ISO 8601 date string
 * @returns Formatted absolute date string
 */
export function formatAbsoluteDate(dateString: string): string {
  const date = new Date(dateString);

  // Handle invalid dates
  if (isNaN(date.getTime())) {
    return "Invalid date";
  }

  const options: Intl.DateTimeFormatOptions = {
    month: "short",
    day: "numeric",
    year: "numeric",
  };

  return date.toLocaleDateString("en-US", options);
}

/**
 * Format a date with time (e.g., "Jan 15, 2025 at 2:30 PM").
 *
 * @param dateString - ISO 8601 date string
 * @returns Formatted date with time string
 */
export function formatDateTime(dateString: string): string {
  const date = new Date(dateString);

  // Handle invalid dates
  if (isNaN(date.getTime())) {
    return "Invalid date";
  }

  const dateOptions: Intl.DateTimeFormatOptions = {
    month: "short",
    day: "numeric",
    year: "numeric",
  };

  const timeOptions: Intl.DateTimeFormatOptions = {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  };

  const dateStr = date.toLocaleDateString("en-US", dateOptions);
  const timeStr = date.toLocaleTimeString("en-US", timeOptions);

  return `${dateStr} at ${timeStr}`;
}

/**
 * Format a time only (e.g., "2:30 PM").
 *
 * @param dateString - ISO 8601 date string
 * @returns Formatted time string
 */
export function formatTime(dateString: string): string {
  const date = new Date(dateString);

  // Handle invalid dates
  if (isNaN(date.getTime())) {
    return "Invalid date";
  }

  const options: Intl.DateTimeFormatOptions = {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  };

  return date.toLocaleTimeString("en-US", options);
}

/**
 * Check if two dates are the same day.
 */
function isSameDay(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
}

/**
 * Truncate text to a maximum length, adding ellipsis if truncated.
 * Attempts to truncate at word boundary when possible.
 *
 * @param text - Text to truncate
 * @param maxLength - Maximum length (default: 100)
 * @returns Truncated text with ellipsis if needed
 */
export function truncateText(text: string | null | undefined, maxLength: number = 100): string {
  // Handle null/undefined/empty input
  if (!text) {
    return "";
  }

  // No truncation needed
  if (text.length <= maxLength) {
    return text;
  }

  // Find last space within limit to truncate at word boundary
  const truncated = text.slice(0, maxLength);
  const lastSpace = truncated.lastIndexOf(" ");

  // If we found a space in the second half, truncate there
  // Otherwise, just truncate at maxLength
  if (lastSpace > maxLength / 2) {
    return truncated.slice(0, lastSpace) + "...";
  }

  return truncated + "...";
}
