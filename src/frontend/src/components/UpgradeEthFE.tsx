const UpgradeEthFE = (props: any) => {

    return <div className="w-full">
        <label className="block mb-2 text-lg text-gray-400">Upgrade Batch Id:</label>
        <input value={props.batchId} onChange={(e) => props.setBatchId(e.target.value)} className="w-full p-3 rounded-lg border border-gray-600 bg-zinc-700 text-white" />
        <label className="block mb-2 text-lg text-gray-400">Upgrade Evidence:</label>
        <input value={props.evidence} onChange={(e) => props.setEvidence(e.target.value)} className="w-full p-3 rounded-lg border border-gray-600 bg-zinc-700 text-white" />
    </div>;
};

export default UpgradeEthFE;