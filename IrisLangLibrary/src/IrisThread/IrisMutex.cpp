#include "IrisThread/IrisMutex.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"


IrisValue IrisMutex::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisMutex::Lock(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {

	if (!static_cast<IrisClosureBlock*>(static_cast<IrisContextEnvironment*>(pContextEnvironment)->GetClosureBlock())) {
		IrisDevUtil::GroanIrregularWithString("Method of lock of a Mutex object need a block to run.");
		return IrisDevUtil::Nil();
	}

	//ivObj.GetIrisObject()->SetPermanent(true);
	auto pMutex = IrisDevUtil::GetNativePointer<IrisMutexTag*>(ivObj);
	pMutex->Lock();
	//IrisGC::SetGCFlag(false);
	static_cast<IrisClosureBlock*>(static_cast<IrisContextEnvironment*>(pContextEnvironment)->GetClosureBlock())->Excute(nullptr);
	//IrisGC::SetGCFlag(true);
	pMutex->Unlock();
	//ivObj.GetIrisObject()->SetPermanent(false);

	return IrisDevUtil::Nil();
}

IrisMutex::IrisMutex()
{
}


IrisMutex::~IrisMutex()
{
}
