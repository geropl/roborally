import React from "react";
import { Player, Register, ESimpleMove, MoveCard, EGamePhaseMap, EGamePhase } from "ts-client/lib/gamestate_pb";
import { ProgramInput, StartPositionInput } from "ts-client/lib/inputs_pb";

export interface ProgramSheetProps {
    player: Player.AsObject;

    gamePhase: EGamePhaseMap[keyof EGamePhaseMap];
    availableStartPositionIds: number[];

    roundId: number; // hack to invalidate component

    onSendProgramInput: (input: ProgramInput) => void;
    onSendStartPosition: (input: StartPositionInput) => void;
}

export interface ProgramSheetState {
    localCards: RegisterCardState[];
    startPositionId?: number;
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

        let inputArea: JSX.Element | undefined = undefined;
        if (this.props.gamePhase === EGamePhase.PREPARATION) {
            inputArea = this.renderStartPositionChooser();
        } else if (this.props.gamePhase === EGamePhase.RUNNING) {
            inputArea = this.renderRegisterProgramSheet();
        }

        return (
            <div key={player.id} style={{ border: '1px solid black'}}>
                <p>Life Tokens: {robot.lifeTokens}</p>
                <p>Damage: {robot.damage}</p>
                {inputArea}
            </div>
        );
    }

    protected renderStartPositionChooser(): JSX.Element {
        const player = this.props.player;
        return (
            <div>
                <input
                    type="text"
                    disabled={!player.inputRequired}
                    value={this.state.startPositionId || ""}
                    onChange={(e) => this.onStartPositionIdChange(e)}/>
                <input type="button" disabled={!player.inputRequired} onClick={() => this.onSendStartPositionClicked()} value="Send" />
                <div>{JSON.stringify(this.props.availableStartPositionIds)}</div>
            </div>
        );
    }

    protected onStartPositionIdChange(event: React.ChangeEvent<HTMLInputElement>) {
        try {
            this.setState({
                startPositionId: Number.parseInt(event.target.value)
            });
        } catch (err) {
        }
    }

    protected renderRegisterProgramSheet(): JSX.Element {
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
            <div>
                <div style={{ display: 'flex', flexDirection: 'row' }}>
                    {registers}
                </div>
                <input type="button" disabled={!inputNeeded} onClick={() => this.onSendProgramInputClicked()} value="Send" />
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

    protected onSendProgramInputClicked() {
        const input = new ProgramInput();
        input.setPlayerId(this.props.player.id);
        input.setRegisterCardsChoicesList(this.state.localCards.map(c => c.moveCardId || 0));
        this.props.onSendProgramInput(input);
    }

    protected onSendStartPositionClicked() {
        const input = new StartPositionInput();
        input.setPlayerId(this.props.player.id);
        input.setStartPositionId(this.state.startPositionId || 0);
        this.props.onSendStartPosition(input);
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