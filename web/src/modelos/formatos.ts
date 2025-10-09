import dayjs from "dayjs";

export function formatDateTimeForServer(
  date: dayjs.Dayjs | null | undefined): string | null {
  if (!date) return null;

  return dayjs(date).format('YYYY-MM-DDTHH:mm:ss');
}

export function formatDateForServer(
  date: dayjs.Dayjs | null | undefined): string | null {
  if (!date) return null;

  return dayjs(date).format('YYYY-MM-DD');
}

export function formatTimeForServer(
  date: dayjs.Dayjs | null | undefined): string | null {
  if (!date) return null;

  return dayjs(date).format('HH:mm');
}

export function dateToStr(date: dayjs.Dayjs) {
  return date.format('DD/MM/YYYY')
}

export function createDayjsFromTime(
  baseDate: dayjs.Dayjs,
  time: string) {
  const timeSplit = time.split(':');

  return baseDate.hour(parseInt(timeSplit[0]))
    .minute(parseInt(timeSplit[1]))
    .second(0)
    .millisecond(0);
}