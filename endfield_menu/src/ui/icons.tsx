import type { ReactNode, SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

function Icon({ children, ...props }: IconProps & { children: ReactNode }) {
  return (
    <svg fill="currentColor" stroke="none" aria-hidden {...props}>
      {children}
    </svg>
  );
}

export function IconGear(props: IconProps) {
  return (
    <Icon viewBox="0 0 24 24" {...props}>
      <path d="M19.14 12.94c.04-.3.06-.61.06-.94s-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z" />
    </Icon>
  );
}

export function IconMail(props: IconProps) {
  return (
    <Icon viewBox="0 0 24 24" {...props}>
      <path d="M3.4 9.6 12 4.32l8.6 5.28z" />
      <path d="M3.4 10.1h17.2v9.5H3.4z" />
      <path d="M3.9 10.1h16.2L12 15.64z" fill="#14171a" />
      <path d="M12 12.94 7.03 17.91 12 15.64z" fill="#14171a" />
    </Icon>
  );
}

export function IconWarn(props: IconProps) {
  return (
    <Icon viewBox="0 0 24 24" {...props}>
      <path fillRule="evenodd" d="M12 2.4 23.2 21.4H.8Z M10.85 9h2.3l-.35 5.6h-1.6Z M10.85 16.4h2.3v2.2h-2.3Z" />
    </Icon>
  );
}

export function IconPower(props: IconProps) {
  return (
    <Icon viewBox="0 0 48 48" {...props}>
      <path fillRule="evenodd" d="M17.04 4.09A21.09 21.09 0 1 0 30.96 4.09V11.43A14.37 14.37 0 1 1 17.04 11.43Z M19.48 0.75H28.08V26.9H19.48Z" />
    </Icon>
  );
}

export function IconX(props: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" aria-hidden {...props}>
      <path d="m5 5 14 14M19 5 5 19" />
    </svg>
  );
}

export function IconSignal(props: IconProps) {
  return (
    <Icon viewBox="0 0 16 14" {...props}>
      <path d="M0 10h3v4H0ZM4.3 7h3v7h-3ZM8.6 3.5h3V14h-3ZM12.9 0H16v14h-3.1Z" />
    </Icon>
  );
}

export function IconBolt(props: IconProps) {
  return (
    <Icon viewBox="0 0 24 24" {...props}>
      <path d="M13 2 4.5 13.5H11L9.5 22 19 10h-6.5Z" />
    </Icon>
  );
}
