import type { Ref } from "react";
import type { Phase } from "../scene/engine";
import { Cables, Cards, NewsBelt } from "./cards";
import { Badge, CloseButton, DiscButtons, Energy, LevelPill, News, PlusMarks, PowerButton, SysButtons, TopBar } from "./chrome";
import { Tiles } from "./tiles";
import { WellMap } from "./well-map";

export function MenuLayers({ camRef, phase }: { camRef: Ref<HTMLDivElement>; phase: Phase }) {
  return (
    <div className="cam" ref={camRef}>
      <WellMap />
      <div className="layer" data-layer="deck">
        <Cables />
        <NewsBelt />
      </div>
      <div className="layer" data-layer="btns">
        <Energy />
        <DiscButtons />
      </div>
      <div className="layer" data-layer="tiles">
        <Tiles />
        <CloseButton />
      </div>
      <div className="layer" data-layer="cards">
        <SysButtons />
        <PowerButton />
        <Cards />
        <Badge />
      </div>
      <div className="layer" data-layer="news">
        <News phase={phase} />
      </div>
      <div className="layer" data-layer="top">
        <TopBar />
      </div>
      <div className="level-anchor" data-anchor="level">
        <LevelPill />
      </div>
      <PlusMarks />
    </div>
  );
}
