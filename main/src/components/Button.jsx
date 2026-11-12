import clsx from "clsx";
import { Link } from "react-router-dom";

const Button = ({
  id,
  title,
  rightIcon,
  leftIcon,
  containerClass,
  to,
  href,
  onClick,
  type = "button",
  target,
  rel,
}) => {
  const className = clsx(
    "group relative z-10 w-fit cursor-pointer overflow-hidden rounded-full bg-violet-50 px-7 py-3 text-black",
    containerClass
  );

  const content = (
    <>
      {leftIcon}
      <span className="relative inline-flex overflow-hidden font-general text-xs uppercase">
        <div className="translate-y-0 skew-y-0 transition duration-500 group-hover:translate-y-[-160%] group-hover:skew-y-12">
          {title}
        </div>
        <div className="absolute translate-y-[164%] skew-y-12 transition duration-500 group-hover:translate-y-0 group-hover:skew-y-0">
          {title}
        </div>
      </span>
      {rightIcon}
    </>
  );

  if (to) {
    return (
      <Link id={id} to={to} className={className} onClick={onClick}>
        {content}
      </Link>
    );
  }

  if (href) {
    return (
      <a
        id={id}
        href={href}
        className={className}
        onClick={onClick}
        target={target}
        rel={rel}
      >
        {content}
      </a>
    );
  }

  return (
    <button id={id} type={type} className={className} onClick={onClick}>
      {content}
    </button>
  );
};

export default Button;
