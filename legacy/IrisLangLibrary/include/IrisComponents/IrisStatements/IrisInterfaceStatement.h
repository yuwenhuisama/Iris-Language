#ifndef _H_IRISINTERFACESTATEMENT_
#define _H_IRISINTERFACESTATEMENT_

#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisBlock;
class IrisExpression;
class IrisInterfaceStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pInterfaceName = nullptr;
	IrisList<IrisExpression*>* m_pInterfaces = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisInterfaceStatement(IrisIdentifier* pInterfaceName, IrisList<IrisExpression*>* pInterfaces, IrisBlock* pBlock);
	virtual ~IrisInterfaceStatement();

	virtual bool Validate() override;
};

#endif