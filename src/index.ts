import { Journal, JournalOptions } from "@kayahr/ed-journal";
import PushBullet from "pushbullet";
import dotenv from "dotenv";
dotenv.config();
const pusher = new PushBullet(process.env.PUSHBULLETKEY);

function sendNotification(device, title: String, body: String) {
    pusher.note(device, title, body)
}


console.log("Running...");
sendNotification({}, "ED AFK Notifier", "ED AFK Notifier is running!");
const JournalOptions: JournalOptions = {
    watch: true,
    position: "end"
}
const journal = await Journal.open(JournalOptions);
try {
    for await (const event of journal) {
        if (event.event === "ShieldState") {
            if (!event.ShieldsUp) {
                sendNotification({}, "Shields Are Down", "Shields are down, Commander!")
                console.log(event.timestamp + " Shields are down, Commander!")
            }
        } else if (event.event === "HullDamage" && event.PlayerPilot) {
            const hullPercentage = event.Health * 100;
            if (hullPercentage < 75 || hullPercentage < 50 || hullPercentage < 25) {
                sendNotification({}, "Hull Damage", `Hull damage detected, Commander! Hull is at ${hullPercentage}%`);
                console.log(`${event.timestamp} Hull damage detected, Commander! Hull is at ${hullPercentage}%`);
            }
        } else if (event.event === "FighterDestroyed") {
            sendNotification({}, "Fighter Destroyed", "Fighter destroyed, Commander!")
            console.log(event.timestamp + ": Fighter destroyed, Commander!")
        } else if (event.event === "Missions") {
            if (event.Active.length === 0) {
                console.log(event.timestamp + ": No active missions, Commander!")
                sendNotification({}, "No Active Missions", "No active missions, Commander!");
            } else {
                console.log(event.Active.length + " Active missions, Commander!")
            }
        } else if (event.event === "ReceiveText") {
            if (event.From_Localised === "System Authority Vessel" && (event.Message.includes("Police_Attack") || event.Message.includes("OverwatchAttackRun"))) { //Added due to a bug in ED where police might attack you for no reason. Can be removed if fixed.
                console.log("Police attack detected, Commander!")
                sendNotification({}, "Police Attack", "Police attack detected, Commander!");
            }
        } else if (event.event === "CollectCargo") {
            if (event.Stolen) {
                console.log("Stolen cargo collected, Commander!")
                sendNotification({}, "Stolen Cargo Collected", "Stolen cargo collected, Commander!");
            }
        } else if (event.event === "Died") {
            console.log("Commander has died!")
            sendNotification({}, "Commander has died!", "Commander has died!");
        }
    };
} catch (err) {
    console.error(err);
    sendNotification({}, "ED AFK Notifier", "ED AFK Notifier has crashed!");
} finally {
    await journal.close();
}
