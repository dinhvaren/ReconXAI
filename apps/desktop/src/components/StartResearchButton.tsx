import { Button } from "./controls";
import { Icon } from "./ui";
import { useWorkspace } from "../workspace";

export default function StartResearchButton({
  projectId,
  targetId,
  providerId,
}: {
  projectId?: string;
  targetId?: string;
  providerId?: string;
}) {
  const workspace = useWorkspace();
  return (
    <Button
      className="button primary"
      disabled={!!workspace?.saving}
      disabledReason="Saving workspace selection"
      onClick={() => {
        const open = () =>
          window.dispatchEvent(
            new CustomEvent("reconx-start-research", {
              detail: { providerId },
            }),
          );
        if (workspace && projectId && targetId)
          void workspace
            .select(projectId, targetId)
            .then(open)
            .catch(() => {});
        else open();
      }}
    >
      <Icon name="workflow" size={16} />
      Start AI Research
    </Button>
  );
}
