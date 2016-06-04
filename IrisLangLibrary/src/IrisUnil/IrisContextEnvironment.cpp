#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisInterpreter.h"
#include "IrisInterfaces/IIrisClosureBlock.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"

IrisContextEnvironment::IrisContextEnvironment() : m_mpVariables()
{
}

IIrisClosureBlock * IrisContextEnvironment::GetClosureBlock() { return m_pClosureBlock; }

void IrisContextEnvironment::SetClosureBlock(IIrisClosureBlock * pBlock) { m_pClosureBlock = static_cast<IrisClosureBlock*>(pBlock); }

IIrisContextEnvironment * IrisContextEnvironment::GetUpperContextEnvrioment() { return m_pUpperContextEnvironment; }

const IrisValue& IrisContextEnvironment::GetVariableValue(const IrisInternString& strVariableName, bool& bResult) {
	decltype(m_mpVariables)::iterator iVariable;
	m_iwlVariableLock.ReadLock();
	if ((iVariable = m_mpVariables.find(strVariableName)) == m_mpVariables.end()){
		bResult = false;
		m_iwlVariableLock.ReadUnlock();
		return IrisInterpreter::CurrentInterpreter()->Nil();
	}
	bResult = true;
	auto& ivResult = iVariable->second;
	m_iwlVariableLock.ReadUnlock();
	return ivResult;
}

void IrisContextEnvironment::AddLocalVariable(const IrisInternString& strVariableName, const IrisValue& ivValue){
	m_iwlVariableLock.WriteLock();
	m_mpVariables.insert(_VariablePair(strVariableName, ivValue));
	m_iwlVariableLock.WriteUnlock();
}


IrisContextEnvironment::~IrisContextEnvironment()
{
}
