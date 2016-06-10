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

#ifdef IR_USE_STL_STRING
const IrisValue& IrisContextEnvironment::GetVariableValue(const string& strVariableName, bool& bResult) {
#else
const IrisValue& IrisContextEnvironment::GetVariableValue(const IrisInternString& strVariableName, bool& bResult) {
#endif // IR_USE_STL_STRING
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

#ifdef IR_USE_STL_STRING
void IrisContextEnvironment::AddLocalVariable(const string& strVariableName, const IrisValue& ivValue) {
#else
void IrisContextEnvironment::AddLocalVariable(const IrisInternString& strVariableName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
	m_iwlVariableLock.WriteLock();
	m_mpVariables.insert(_VariablePair(strVariableName, ivValue));
	m_iwlVariableLock.WriteUnlock();
}


IrisContextEnvironment::~IrisContextEnvironment()
{
}
