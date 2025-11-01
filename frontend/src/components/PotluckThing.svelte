<div class="pico potluck-planner center">
    {#if state == State.FindOrCreate}
        <h1><center>Potluck Planner</center></h1>
        <div> 
            <!-- svelte-ignore a11y-no-redundant-roles -->
            <fieldset role="group">
                <input type="text" placeholder="Party Code" bind:value={searchUUID}/>
                <button on:click={() => findParty(searchUUID)}>Find</button>
            </fieldset>
            <center class="even"> or </center>
            <center><button on:click={createNew}>Create New</button></center>
        </div>
    {:else if state == State.Error}
        <button on:click={reset}>Back</button>
        <p class="error">Error: {error}</p>
    {:else if state == State.CreateNew}
        <button on:click={reset}>Back</button>
        <form>
            <h1><center>Create A Party</center></h1>
            <input type="text" placeholder="Party Name" bind:value={partyName}/>
            <input type="date" bind:value={date}/>
            <input type="time" bind:value={time} placeholder="time"/>
            <input type="text" placeholder="Location" bind:value={location}/>
            <center><button on:click = {createParty}>Create</button></center>
        </form>
    {:else if state == State.PartyCreated}
        <center>    
            <h1>Success!</h1>
            <h2>Your party's code is below, save it!</h2>
            <h2>{newUUID}</h2>
            <button on:click={()=>findParty(newUUID)}>View</button>
        </center>
    {:else if state == State.Party}
        <button on:click={reset}>Back</button>
        <center>
            <h1>{partyName}</h1>
            <table>
                <thead>
                    <tr>
                        <td>Participant</td>
                        <td>Food Item</td>
                    </tr>   
                </thead>
                <tbody>
                    {#each items as item}
                        <tr>
                            <td>{item.name}</td>
                            <td>{item.item}</td>
                        </tr>
                    {/each}
                    
                </tbody>
            </table>

            <input type="text" placeholder="Your name"/>
            <input type="text" placeholder="What you're bringing (can't edit later!)"/>
            <button>Commit</button>
            
            <center><small>{partyUUID}</small></center>

        </center>
    {/if}
</div>

<script lang="ts">
    enum State 
    {
        FindOrCreate,
        Error,
        CreateNew,
        PartyCreated,
        Party
    }

    let state : State = State.Party;
    let searchUUID = "";
    let error = "bad things!";
    let partyName = "Fake Party";
    let newUUID = "fake uuid";
    let date : Date;
    let time : any;
    let location = "my house";
    let partyUUID = "abcd";
    let items : {name:string,item:string}[]= [{name: "lizbeth",item : "nachos"}];

    function reset() 
    {
        searchUUID = "";
        state = State.FindOrCreate;
    }

    function findParty(uuid : string){state = State.Party}

    function createNew(){state = State.CreateNew;}

    function createParty(){state = State.PartyCreated}

    function commit(){}

</script>

<style>
    .potluck-planner
    {
        max-width: 576px;
        margin: 0 auto 0 auto;
        padding:1rem;
        box-sizing: border-box;
        border-radius: 1rem;
        border: 1.5px solid gray;
    }

    .even{margin-bottom:16px;}
    .error{color: red;}
    input.no-margin{margin:0;}
    input.left16{position:relative; padding-left:8px; right:8px; width:100%;}
</style>