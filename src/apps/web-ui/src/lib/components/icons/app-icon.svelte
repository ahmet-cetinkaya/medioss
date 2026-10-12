<script lang="ts">
  import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";
  import ArrowCounterClockwiseIcon from "phosphor-svelte/lib/ArrowCounterClockwiseIcon";
  import ArrowDownIcon from "phosphor-svelte/lib/ArrowDownIcon";
  import ArrowLeftIcon from "phosphor-svelte/lib/ArrowLeftIcon";
  import ArrowUpIcon from "phosphor-svelte/lib/ArrowUpIcon";
  import ArrowUpRightIcon from "phosphor-svelte/lib/ArrowUpRightIcon";
  import BooksIcon from "phosphor-svelte/lib/BooksIcon";
  import CaretDownIcon from "phosphor-svelte/lib/CaretDownIcon";
  import ChartBarIcon from "phosphor-svelte/lib/ChartBarIcon";
  import CheckCircleIcon from "phosphor-svelte/lib/CheckCircleIcon";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import ClockCounterClockwiseIcon from "phosphor-svelte/lib/ClockCounterClockwiseIcon";
  import DotsThreeIcon from "phosphor-svelte/lib/DotsThreeIcon";
  import DownloadSimpleIcon from "phosphor-svelte/lib/DownloadSimpleIcon";
  import GearSixIcon from "phosphor-svelte/lib/GearSixIcon";
  import HouseIcon from "phosphor-svelte/lib/HouseIcon";
  import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
  import PauseIcon from "phosphor-svelte/lib/PauseIcon";
  import PencilSimpleIcon from "phosphor-svelte/lib/PencilSimpleIcon";
  import PlayIcon from "phosphor-svelte/lib/PlayIcon";
  import PlusIcon from "phosphor-svelte/lib/PlusIcon";
  import ShieldCheckIcon from "phosphor-svelte/lib/ShieldCheckIcon";
  import ShuffleIcon from "phosphor-svelte/lib/ShuffleIcon";
  import SignOutIcon from "phosphor-svelte/lib/SignOutIcon";
  import SkipBackIcon from "phosphor-svelte/lib/SkipBackIcon";
  import SmileyIcon from "phosphor-svelte/lib/SmileyIcon";
  import SpeakerHighIcon from "phosphor-svelte/lib/SpeakerHighIcon";
  import TimerIcon from "phosphor-svelte/lib/TimerIcon";
  import TrashIcon from "phosphor-svelte/lib/TrashIcon";
  import UploadSimpleIcon from "phosphor-svelte/lib/UploadSimpleIcon";
  import UserIcon from "phosphor-svelte/lib/UserIcon";
  import WarningCircleIcon from "phosphor-svelte/lib/WarningCircleIcon";
  import WaveformIcon from "phosphor-svelte/lib/WaveformIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import BirdIcon from "phosphor-svelte/lib/BirdIcon";
  import CloudLightningIcon from "phosphor-svelte/lib/CloudLightningIcon";
  import CloudRainIcon from "phosphor-svelte/lib/CloudRainIcon";
  import CoffeeIcon from "phosphor-svelte/lib/CoffeeIcon";
  import FireIcon from "phosphor-svelte/lib/FireIcon";
  import HeartbeatIcon from "phosphor-svelte/lib/HeartbeatIcon";
  import MoonStarsIcon from "phosphor-svelte/lib/MoonStarsIcon";
  import MusicNotesIcon from "phosphor-svelte/lib/MusicNotesIcon";
  import TrainIcon from "phosphor-svelte/lib/TrainIcon";
  import TreeIcon from "phosphor-svelte/lib/TreeIcon";
  import WavesIcon from "phosphor-svelte/lib/WavesIcon";
  import WindIcon from "phosphor-svelte/lib/WindIcon";
  import DropIcon from "phosphor-svelte/lib/DropIcon";
  import LeafIcon from "phosphor-svelte/lib/LeafIcon";
  import BellRingingIcon from "phosphor-svelte/lib/BellRingingIcon";
  import PianoKeysIcon from "phosphor-svelte/lib/PianoKeysIcon";
  import GuitarIcon from "phosphor-svelte/lib/GuitarIcon";
  import FanIcon from "phosphor-svelte/lib/FanIcon";
  import AirplaneIcon from "phosphor-svelte/lib/AirplaneIcon";
  import BugIcon from "phosphor-svelte/lib/BugIcon";
  import CatIcon from "phosphor-svelte/lib/CatIcon";
  import MountainsIcon from "phosphor-svelte/lib/MountainsIcon";
  import CampfireIcon from "phosphor-svelte/lib/CampfireIcon";
  import SnowflakeIcon from "phosphor-svelte/lib/SnowflakeIcon";
  import type { IconName } from "./icon-name";

  const ICONS: Record<IconName, typeof HouseIcon> = {
    house: HouseIcon,
    books: BooksIcon,
    waveform: WaveformIcon,
    smiley: SmileyIcon,
    "clock-counter-clockwise": ClockCounterClockwiseIcon,
    "chart-bar": ChartBarIcon,
    "gear-six": GearSixIcon,
    "shield-check": ShieldCheckIcon,
    shuffle: ShuffleIcon,
    "sign-out": SignOutIcon,
    user: UserIcon,
    play: PlayIcon,
    pause: PauseIcon,
    "arrow-counter-clockwise": ArrowCounterClockwiseIcon,
    "arrow-clockwise": ArrowClockwiseIcon,
    "arrow-up-right": ArrowUpRightIcon,
    "arrow-up": ArrowUpIcon,
    "arrow-down": ArrowDownIcon,
    "arrow-left": ArrowLeftIcon,
    "download-simple": DownloadSimpleIcon,
    "upload-simple": UploadSimpleIcon,
    "magnifying-glass": MagnifyingGlassIcon,
    check: CheckIcon,
    "check-circle": CheckCircleIcon,
    "warning-circle": WarningCircleIcon,
    trash: TrashIcon,
    "pencil-simple": PencilSimpleIcon,
    x: XIcon,
    plus: PlusIcon,
    "dots-three": DotsThreeIcon,
    "caret-down": CaretDownIcon,
    timer: TimerIcon,
    "skip-back": SkipBackIcon,
    "speaker-high": SpeakerHighIcon,
    birds: BirdIcon,
    thunder: CloudLightningIcon,
    rain: CloudRainIcon,
    noise: CoffeeIcon,
    fire: FireIcon,
    heartbeat: HeartbeatIcon,
    night: MoonStarsIcon,
    music: MusicNotesIcon,
    city: TrainIcon,
    forest: TreeIcon,
    water: WavesIcon,
    wind: WindIcon,
    drop: DropIcon,
    leaf: LeafIcon,
    bell: BellRingingIcon,
    piano: PianoKeysIcon,
    guitar: GuitarIcon,
    fan: FanIcon,
    airplane: AirplaneIcon,
    cricket: BugIcon,
    cat: CatIcon,
    mountains: MountainsIcon,
    campfire: CampfireIcon,
    snowflake: SnowflakeIcon,
  };

  type Props = {
    name: IconName;
    size?: 22 | 20 | 16 | 14;
    filled?: boolean;
    label?: string;
    class?: string;
  };

  let {
    name,
    size = 22,
    filled = false,
    label,
    class: className,
  }: Props = $props();

  const Icon = $derived(ICONS[name]);
</script>

{#if label}
  <Icon
    {size}
    weight={filled ? "fill" : "regular"}
    class={className}
    role="img"
    aria-label={label}
  />
{:else}
  <Icon
    {size}
    weight={filled ? "fill" : "regular"}
    class={className}
    role={undefined}
    aria-hidden="true"
    focusable="false"
  />
{/if}
