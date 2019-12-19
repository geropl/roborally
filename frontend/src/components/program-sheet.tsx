import React from "react";
import { Player, Register, ESimpleMove, MoveCard } from "ts-client/lib/gamestate_pb";
import { PlayerInput } from "ts-client/lib/inputs_pb";

export interface ProgramSheetProps {
    roundId: number; // hack to invalidate component
    player: Player.AsObject;
    onPlayerInputClicked: (input: PlayerInput) => void;
}

export interface ProgramSheetState {
    localCards: RegisterCardState[];
}

interface RegisterCardState {
    moveCardId?: number;
    moveCard?: MoveCard.AsObject;
}

export class ProgramSheet extends React.Component<ProgramSheetProps, ProgramSheetState> {

    constructor(props: ProgramSheetProps) {
        super(props);

        const localCards = [];
        for (let i = 0; i < 5; i++) {
            localCards.push({});
        }
        this.state = { localCards };
    }

    render() {
        const player = this.props.player;
        const robot = player.robot!;
        
        const inputNeeded = robot.damage < 9
            && robot.lifeTokens > 0
            && player.registersList.reduce((acc, r) => acc + (r.locked ? 0 : 1), 0) > 0;   // TODO Judge in backend

        let registerId = 0;
        const registers = player.registersList.map(r => {
            const id = registerId++;
            const local = this.state.localCards[id];
            return (
                <RegisterCard
                    id={id}
                    register={r}
                    onMoveCardIdChange={(id, event) => this.onMoveCardIdChange(id, event)}
                    local={local} />
            );
        });

        return (
            <div key={player.id} style={{ border: '1px solid black'}}>
                <p>Life Tokens: {robot.lifeTokens}</p>
                <p>Damage: {robot.damage}</p>
                <div style={{ display: 'flex', flexDirection: 'row' }}>
                    {registers}
                </div>
                <input type="button" disabled={!inputNeeded} onClick={() => this.onSendClicked()} value="Send" />
                <div>{JSON.stringify(player.programCardDeckList)}</div>
            </div>
        );
    }

    protected onMoveCardIdChange(id: number, event: React.ChangeEvent<HTMLInputElement>) {
        try {
            const moveCardId = Number.parseInt(event.target.value);
            const moveCard = this.props.player.programCardDeckList.find(c => c.id === moveCardId);

            this.setState(os => {
                os.localCards[id] = {
                    moveCardId,
                    moveCard
                };
                return os;
            });
        } catch (err) {
            this.setState(os => {
                os.localCards[id] = {
                    moveCard: undefined
                };
                return os;
            });
        }
    }

    protected onSendClicked() {
        const input = new PlayerInput();
        input.setPlayerId(this.props.player.id);
        input.setRegisterCardsChoicesList(this.state.localCards.map(c => c.moveCardId || 0));
        this.props.onPlayerInputClicked(input);
    }
}

interface RegisterCardProps {
    id: number;
    register: Register.AsObject;

    onMoveCardIdChange: (id: number, event: React.ChangeEvent<HTMLInputElement>) => void;
    local: RegisterCardState;
}

class RegisterCard extends React.Component<RegisterCardProps> {
    render() {
        const r = this.props.register;
        const renderedCard = this.renderMoveCard(r.moveCard || this.props.local.moveCard);
        return (
            <div key={this.props.id}>
                <h3>Register {this.props.id}</h3>
                <p>Locked: {JSON.stringify(r.locked)}</p>
                <input
                    type="text"
                    disabled={r.locked || r.moveCard !== undefined}
                    value={this.props.local.moveCardId || ""}
                    onChange={(e) => this.props.onMoveCardIdChange(this.props.id, e)}/>
                {renderedCard}
            </div>
        );
    }

    protected renderMoveCard(moveCard?: MoveCard.AsObject): JSX.Element {
        if (!moveCard) {
            return <span>not-set</span>;
        }

        return (
            <div>
                <p>Priority: {moveCard.priority}</p>
                <p>{JSON.stringify(moveCard.movesList.map(i => Object.keys(ESimpleMove)[i]))}</p>
            </div>
        );
    }
}